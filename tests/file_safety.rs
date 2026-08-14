use tempfile::TempDir;
use tmpmail::files::{atomic_write, safe_basename};

#[test]
fn rejects_traversal_controls_and_reserved_basenames() {
    for value in ["../mail", "a/b", "a\\b", "CON", "con.txt", "bad\u{1b}"] {
        assert!(safe_basename(value).is_err(), "accepted {value:?}");
    }
    assert_eq!(safe_basename("report 1.txt").expect("safe"), "report 1.txt");
}

#[test]
fn atomic_output_refuses_symlink_and_cleans_temporary_files() {
    let temp = TempDir::new().expect("tempdir");
    let target = temp.path().join("message.html");
    atomic_write(&target, b"safe", true).expect("write");
    assert_eq!(std::fs::read(&target).expect("read"), b"safe");
    #[cfg(unix)]
    {
        let link = temp.path().join("link.html");
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        assert_eq!(
            atomic_write(&link, b"unsafe", true)
                .expect_err("refuse")
                .code(),
            "unsafe_path"
        );
    }
    assert_eq!(
        std::fs::read_dir(temp.path()).expect("list").count(),
        if cfg!(unix) { 2 } else { 1 }
    );
}

#[test]
fn non_replacing_output_uses_an_atomic_destination_claim() {
    let temp = TempDir::new().expect("tempdir");
    let target = temp.path().join("attachment.txt");
    std::fs::write(&target, b"existing").expect("fixture");
    let error = atomic_write(&target, b"replacement", false).expect_err("exclusive claim");
    assert_eq!(error.code(), "unsafe_path");
    assert_eq!(std::fs::read(&target).expect("unchanged"), b"existing");
    assert_eq!(std::fs::read_dir(temp.path()).expect("list").count(), 1);
}
