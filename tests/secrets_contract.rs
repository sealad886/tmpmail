use tempfile::TempDir;
use tmpmail::secrets::{ProtectedFileSecretStore, SecretStore};

#[test]
fn protected_file_round_trips_without_plaintext_reference_leaks() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("secrets.json");
    let store = ProtectedFileSecretStore::at(&path);
    store.put("inbox:work", b"secret-value").expect("store");
    assert_eq!(
        store.get("inbox:work").expect("read"),
        Some(b"secret-value".to_vec())
    );
    store.remove("inbox:work").expect("remove");
    assert_eq!(store.get("inbox:work").expect("read"), None);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o077,
            0
        );
    }
}

#[test]
fn fallback_file_is_not_created_until_explicitly_used() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("secrets.json");
    let _store = ProtectedFileSecretStore::at(&path);
    assert!(!path.exists());
}
