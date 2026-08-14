use crate::error::AppError;
use std::collections::BTreeMap;
use std::sync::RwLock;

pub trait SecretStore: Send + Sync {
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError>;
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError>;
    fn remove(&self, reference: &str) -> Result<(), AppError>;
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
        match Self::entry(reference)?.get_secret() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AppError::new(
                "keyring_read",
                "could not read credentials from OS credential store",
                4,
            )),
        }
    }

    #[cfg(all(feature = "native-keyring", target_os = "macos"))]
    fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        macos_keychain::get(reference)
    }

    #[cfg(any(
        not(feature = "native-keyring"),
        not(any(target_os = "macos", target_os = "linux"))
    ))]
    fn get(&self, _reference: &str) -> Result<Option<Vec<u8>>, AppError> {
        Err(keyring_disabled())
    }

    #[cfg(all(feature = "native-keyring", target_os = "linux"))]
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        Self::entry(reference)?.set_secret(secret).map_err(|_| {
            AppError::new(
                "keyring_write",
                "could not store credentials in OS credential store",
                4,
            )
        })
    }

    #[cfg(all(feature = "native-keyring", target_os = "macos"))]
    fn put(&self, reference: &str, secret: &[u8]) -> Result<(), AppError> {
        macos_keychain::put(reference, secret)
    }

    #[cfg(any(
        not(feature = "native-keyring"),
        not(any(target_os = "macos", target_os = "linux"))
    ))]
    fn put(&self, _reference: &str, _secret: &[u8]) -> Result<(), AppError> {
        Err(keyring_disabled())
    }

    #[cfg(all(feature = "native-keyring", target_os = "linux"))]
    fn remove(&self, reference: &str) -> Result<(), AppError> {
        match Self::entry(reference)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(AppError::new(
                "keyring_delete",
                "could not remove credentials from OS credential store",
                4,
            )),
        }
    }

    #[cfg(all(feature = "native-keyring", target_os = "macos"))]
    fn remove(&self, reference: &str) -> Result<(), AppError> {
        macos_keychain::remove(reference)
    }

    #[cfg(any(
        not(feature = "native-keyring"),
        not(any(target_os = "macos", target_os = "linux"))
    ))]
    fn remove(&self, _reference: &str) -> Result<(), AppError> {
        Err(keyring_disabled())
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
