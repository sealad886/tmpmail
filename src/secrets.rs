use crate::error::AppError;
use fs2::FileExt;
use rand::distr::{Alphanumeric, SampleString};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::RwLock;

pub trait SecretStore: Send + Sync {
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError>;
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError>;
    fn remove(&self, reference: &str) -> Result<(), AppError>;
}

pub fn unique_inbox_reference(provider: crate::core::ProviderId) -> String {
    let nonce = Alphanumeric.sample_string(&mut rand::rng(), 24);
    format!("inbox:{provider}:{nonce}")
}

#[derive(Debug, Default)]
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    #[cfg(all(feature = "native-keyring", target_os = "linux"))]
    fn entry(reference: &str) -> Result<keyring::Entry, AppError> {
        keyring::Entry::new("tmpmail", reference).map_err(|_| {
            AppError::new(
                "keyring_unavailable",
                "OS credential store is unavailable; no secret was stored",
                4,
            )
        })
    }
}

impl SecretStore for KeyringSecretStore {
    #[cfg(all(feature = "native-keyring", target_os = "linux"))]
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        let native = Self::entry(reference).and_then(|entry| match entry.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AppError::new(
                "keyring_read",
                "could not read credentials from OS credential store",
                4,
            )),
        });
        fallback_get(reference, native)
    }

    #[cfg(all(feature = "native-keyring", target_os = "macos"))]
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        fallback_get(reference, macos_keychain::get(reference))
    }

    #[cfg(any(
        not(feature = "native-keyring"),
        not(any(target_os = "macos", target_os = "linux"))
    ))]
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        fallback_get(reference, Err(keyring_disabled()))
    }

    #[cfg(all(feature = "native-keyring", target_os = "linux"))]
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        let native = Self::entry(reference).and_then(|entry| {
            entry.set_secret(secret).map_err(|_| {
                AppError::new(
                    "keyring_write",
                    "could not store credentials in OS credential store",
                    4,
                )
            })
        });
        fallback_put(reference, secret, native)
    }

    #[cfg(all(feature = "native-keyring", target_os = "macos"))]
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        let native = macos_keychain::put(reference, secret);
        fallback_put(reference, secret, native)
    }

    #[cfg(any(
        not(feature = "native-keyring"),
        not(any(target_os = "macos", target_os = "linux"))
    ))]
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        fallback_put(reference, secret, Err(keyring_disabled()))
    }

    #[cfg(all(feature = "native-keyring", target_os = "linux"))]
    fn remove(&self, reference: &str) -> Result<(), AppError> {
        let native = Self::entry(reference).and_then(|entry| match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(AppError::new(
                "keyring_delete",
                "could not remove credentials from OS credential store",
                4,
            )),
        });
        fallback_remove(reference, native)
    }

    #[cfg(all(feature = "native-keyring", target_os = "macos"))]
    fn remove(&self, reference: &str) -> Result<(), AppError> {
        let native = macos_keychain::remove(reference);
        fallback_remove(reference, native)
    }

    #[cfg(any(
        not(feature = "native-keyring"),
        not(any(target_os = "macos", target_os = "linux"))
    ))]
    fn remove(&self, reference: &str) -> Result<(), AppError> {
        fallback_remove(reference, Err(keyring_disabled()))
    }
}

#[derive(Debug, Clone)]
pub struct ProtectedFileSecretStore {
    path: PathBuf,
}

impl ProtectedFileSecretStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn load(&self) -> Result<BTreeMap<String, Vec<u8>>, AppError> {
        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&self.path)
                .map_err(|_| secret_file_error())?
                .permissions()
                .mode()
                & 0o077
                != 0
            {
                return Err(AppError::new(
                    "secret_file_permissions",
                    "secret file permissions are not 0600",
                    4,
                ));
            }
        }
        let bytes = fs::read(&self.path).map_err(|_| secret_file_error())?;
        serde_json::from_slice(&bytes).map_err(|_| secret_file_error())
    }

    fn save(&self, values: &BTreeMap<String, Vec<u8>>) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|_| secret_file_error())?;
        }
        let temporary = self
            .path
            .with_extension(format!("json.{}.tmp", std::process::id()));
        let bytes = serde_json::to_vec(values).map_err(|_| secret_file_error())?;
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary).map_err(|_| secret_file_error())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| secret_file_error())?;
        fs::rename(&temporary, &self.path).map_err(|_| secret_file_error())?;
        if let Some(parent) = self.path.parent()
            && let Ok(directory) = fs::File::open(parent)
        {
            directory.sync_all().map_err(|_| secret_file_error())?;
        }
        Ok(())
    }

    fn lock(&self) -> Result<std::fs::File, AppError> {
        let parent = self.path.parent().ok_or_else(secret_file_error)?;
        fs::create_dir_all(parent).map_err(|_| secret_file_error())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.path.with_extension("lock"))
            .map_err(|_| secret_file_error())?;
        lock.lock_exclusive().map_err(|_| secret_file_error())?;
        Ok(lock)
    }
}

impl SecretStore for ProtectedFileSecretStore {
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        Ok(self.load()?.get(reference).cloned())
    }
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        let lock = self.lock()?;
        let mut values = self.load()?;
        values.insert(reference.to_owned(), secret.to_vec());
        let result = self.save(&values);
        let _ = FileExt::unlock(&lock);
        result
    }
    fn remove(&self, reference: &str) -> Result<(), AppError> {
        let lock = self.lock()?;
        let mut values = self.load()?;
        values.remove(reference);
        let result = self.save(&values);
        let _ = FileExt::unlock(&lock);
        result
    }
}

fn fallback_store() -> Option<ProtectedFileSecretStore> {
    let paths = crate::paths::AppPaths::discover().ok()?;
    let config = crate::config::ConfigStore::at(&paths.config).load().ok()?;
    config
        .secrets_file_fallback
        .then(|| ProtectedFileSecretStore::at(paths.state.with_file_name("secrets.json")))
}

fn fallback_get(
    reference: &str,
    native: Result<Option<Vec<u8>>, AppError>,
) -> Result<Option<Vec<u8>>, AppError> {
    match native {
        Ok(Some(secret)) => Ok(Some(secret)),
        Ok(None) => fallback_store().map_or(Ok(None), |store| store.get(reference)),
        Err(error) => fallback_store().map_or(Err(error), |store| store.get(reference)),
    }
}

fn fallback_put(
    reference: &str,
    secret: &[u8],
    native: Result<(), AppError>,
) -> Result<(), AppError> {
    let fallback = fallback_store();
    fallback_put_to(reference, secret, native, fallback.as_ref())
}

fn fallback_put_to(
    reference: &str,
    secret: &[u8],
    native: Result<(), AppError>,
    fallback: Option<&ProtectedFileSecretStore>,
) -> Result<(), AppError> {
    match native {
        Ok(()) => {
            if let Some(store) = fallback {
                store.remove(reference).map_err(|_| {
                    AppError::new(
                        "partial_operation",
                        "credential changed in the OS store but stale protected-file cleanup failed",
                        10,
                    )
                })?;
            }
            Ok(())
        }
        Err(error) => fallback.map_or(Err(error), |store| store.put(reference, secret)),
    }
}

fn fallback_remove(reference: &str, native: Result<(), AppError>) -> Result<(), AppError> {
    match native {
        Ok(()) => {
            if let Some(store) = fallback_store() {
                store.remove(reference).map_err(|_| {
                    AppError::new(
                        "partial_operation",
                        "credential was removed from the OS store but protected-file cleanup failed",
                        10,
                    )
                })?;
            }
            Ok(())
        }
        Err(error) => fallback_store().map_or(Err(error), |store| store.remove(reference)),
    }
}

fn secret_file_error() -> AppError {
    AppError::new("secret_file", "protected secret file operation failed", 4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_rotation_removes_stale_file_fallback() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let store = ProtectedFileSecretStore::at(temp.path().join("secrets.json"));
        store.put("token", b"old").expect("old fallback");
        fallback_put_to("token", b"new", Ok(()), Some(&store)).expect("native rotation");
        assert_eq!(store.get("token").expect("fallback read"), None);
    }

    #[test]
    fn native_rotation_reports_partial_state_when_fallback_cleanup_fails() {
        let temp = tempfile::TempDir::new().expect("tempdir");
        let directory = temp.path().join("secrets.json");
        std::fs::create_dir(&directory).expect("blocking directory");
        let store = ProtectedFileSecretStore::at(directory);
        let error =
            fallback_put_to("token", b"new", Ok(()), Some(&store)).expect_err("cleanup must fail");
        assert_eq!(error.code(), "partial_operation");
        assert_eq!(error.exit_code(), 10);
    }
}

#[cfg(not(feature = "native-keyring"))]
fn keyring_disabled() -> AppError {
    AppError::new(
        "keyring_unavailable",
        "this build does not include native credential storage",
        4,
    )
}

#[cfg(all(feature = "native-keyring", target_os = "macos"))]
mod macos_keychain {
    use super::AppError;
    use libloading::{Library, Symbol};
    use std::ffi::c_void;
    use std::ptr;

    const SERVICE: &[u8] = b"tmpmail";
    const ERR_NOT_FOUND: i32 = -25300;
    const ERR_DUPLICATE: i32 = -25299;
    const SECURITY_PATH: &str = "/System/Library/Frameworks/Security.framework/Security";
    const CORE_FOUNDATION_PATH: &str =
        "/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation";

    type Item = *mut c_void;
    type Find = unsafe extern "C" fn(
        *const c_void,
        u32,
        *const c_void,
        u32,
        *const c_void,
        *mut u32,
        *mut *mut c_void,
        *mut Item,
    ) -> i32;
    type Add = unsafe extern "C" fn(
        *const c_void,
        u32,
        *const c_void,
        u32,
        *const c_void,
        u32,
        *const c_void,
        *mut Item,
    ) -> i32;
    type Modify = unsafe extern "C" fn(Item, *const c_void, u32, *const c_void) -> i32;
    type Delete = unsafe extern "C" fn(Item) -> i32;
    type FreeContent = unsafe extern "C" fn(*const c_void, *mut c_void) -> i32;
    type Release = unsafe extern "C" fn(*const c_void);

    pub fn get(reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        // SAFETY: framework paths and symbol signatures are stable macOS C APIs;
        // returned password bytes are copied before provider-owned content is freed.
        unsafe {
            let security = load(SECURITY_PATH)?;
            let find: Symbol<Find> = symbol(&security, b"SecKeychainFindGenericPassword\0")?;
            let free: Symbol<FreeContent> = symbol(&security, b"SecKeychainItemFreeContent\0")?;
            let mut length = 0;
            let mut data = ptr::null_mut();
            let status = find(
                ptr::null(),
                SERVICE.len() as u32,
                SERVICE.as_ptr().cast(),
                reference.len() as u32,
                reference.as_ptr().cast(),
                &mut length,
                &mut data,
                ptr::null_mut(),
            );
            if status == ERR_NOT_FOUND {
                return Ok(None);
            }
            check(status, "read")?;
            let secret = std::slice::from_raw_parts(data.cast::<u8>(), length as usize).to_vec();
            let _ = free(ptr::null(), data);
            Ok(Some(secret))
        }
    }

    pub fn put(reference: &str, secret: &[u8]) -> Result<(), AppError> {
        // SAFETY: all buffers remain alive for each synchronous C call. Item refs
        // returned by Security are released through CoreFoundation.
        unsafe {
            let security = load(SECURITY_PATH)?;
            let add: Symbol<Add> = symbol(&security, b"SecKeychainAddGenericPassword\0")?;
            let status = add(
                ptr::null(),
                SERVICE.len() as u32,
                SERVICE.as_ptr().cast(),
                reference.len() as u32,
                reference.as_ptr().cast(),
                secret.len() as u32,
                secret.as_ptr().cast(),
                ptr::null_mut(),
            );
            if status != ERR_DUPLICATE {
                return check(status, "write");
            }
            let (item, security) = find_item(reference, security)?;
            let modify: Symbol<Modify> =
                symbol(&security, b"SecKeychainItemModifyAttributesAndData\0")?;
            let status = modify(
                item,
                ptr::null(),
                secret.len() as u32,
                secret.as_ptr().cast(),
            );
            release(item)?;
            check(status, "write")
        }
    }

    pub fn remove(reference: &str) -> Result<(), AppError> {
        // SAFETY: item reference comes from Security and is released after use.
        unsafe {
            let security = load(SECURITY_PATH)?;
            let (item, security) = match find_item(reference, security) {
                Ok(found) => found,
                Err(error) if error.code() == "keyring_missing" => return Ok(()),
                Err(error) => return Err(error),
            };
            let delete: Symbol<Delete> = symbol(&security, b"SecKeychainItemDelete\0")?;
            let status = delete(item);
            release(item)?;
            check(status, "delete")
        }
    }

    unsafe fn find_item(reference: &str, security: Library) -> Result<(Item, Library), AppError> {
        let mut item = ptr::null_mut();
        let status = {
            let find: Symbol<Find> =
                unsafe { symbol(&security, b"SecKeychainFindGenericPassword\0")? };
            unsafe {
                find(
                    ptr::null(),
                    SERVICE.len() as u32,
                    SERVICE.as_ptr().cast(),
                    reference.len() as u32,
                    reference.as_ptr().cast(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut item,
                )
            }
        };
        if status == ERR_NOT_FOUND {
            return Err(AppError::new("keyring_missing", "credential not found", 4));
        }
        check(status, "read")?;
        Ok((item, security))
    }

    unsafe fn release(item: Item) -> Result<(), AppError> {
        let core = load(CORE_FOUNDATION_PATH)?;
        let release: Symbol<Release> = unsafe { symbol(&core, b"CFRelease\0")? };
        unsafe { release(item.cast()) };
        Ok(())
    }

    fn load(path: &str) -> Result<Library, AppError> {
        // SAFETY: fixed absolute system framework path, never user-controlled.
        unsafe { Library::new(path) }.map_err(|_| unavailable())
    }

    unsafe fn symbol<'a, T>(library: &'a Library, name: &[u8]) -> Result<Symbol<'a, T>, AppError> {
        unsafe { library.get(name) }.map_err(|_| unavailable())
    }

    fn check(status: i32, operation: &str) -> Result<(), AppError> {
        if status == 0 {
            Ok(())
        } else {
            Err(AppError::new(
                "keyring_failure",
                format!("macOS Keychain {operation} failed (status {status})"),
                4,
            ))
        }
    }

    fn unavailable() -> AppError {
        AppError::new(
            "keyring_unavailable",
            "macOS Keychain framework is unavailable",
            4,
        )
    }
}

#[derive(Debug, Default)]
pub struct MemorySecretStore {
    values: RwLock<BTreeMap<String, Vec<u8>>>,
}

impl SecretStore for MemorySecretStore {
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        Ok(self
            .values
            .read()
            .expect("secret lock poisoned")
            .get(reference)
            .cloned())
    }

    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        self.values
            .write()
            .expect("secret lock poisoned")
            .insert(reference.to_owned(), secret.to_vec());
        Ok(())
    }

    fn remove(&self, reference: &str) -> Result<(), AppError> {
        self.values
            .write()
            .expect("secret lock poisoned")
            .remove(reference);
        Ok(())
    }
}
