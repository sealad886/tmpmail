use std::process::Command;

#[test]
fn invalid_json_invocation_uses_the_versioned_error_envelope() {
    let output = Command::new(env!("CARGO_BIN_EXE_tmpmail"))
        .args(["--json", "not-a-command"])
        .output()
        .expect("run tmpmail");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON error");
    assert_eq!(value["schema"], "tmpmail/v1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "usage");
    assert!(value["meta"]["provider"].is_null());
    assert_eq!(value["meta"]["attempts"], serde_json::json!([]));
}
