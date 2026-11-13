use std::process::Command;

#[test]
fn network_timeout_returns_json_instead_of_panicking() {
    let proxy = std::net::TcpListener::bind("127.0.0.1:0").expect("bind silent proxy");
    let proxy_url = format!("http://{}", proxy.local_addr().expect("proxy address"));
    let temp = tempfile::TempDir::new().expect("isolated paths");
    let output = Command::new(env!("CARGO_BIN_EXE_tmpmail"))
        .args([
            "--json",
            "--timeout",
            "100ms",
            "provider",
            "domains",
            "mailtm",
        ])
        .env("HTTPS_PROXY", &proxy_url)
        .env("https_proxy", &proxy_url)
        .env("NO_PROXY", "")
        .env("no_proxy", "")
        .env("TMPMAIL_CONFIG_DIR", temp.path().join("config"))
        .env("TMPMAIL_STATE_DIR", temp.path().join("state"))
        .env("TMPMAIL_CACHE_DIR", temp.path().join("cache"))
        .output()
        .expect("run tmpmail");
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON timeout");
    assert_eq!(value["schema"], "tmpmail/v1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "provider_timeout");
}

#[test]
fn help_and_version_requests_succeed_with_other_arguments() {
    for arguments in [
        vec!["message", "--help"],
        vec!["inbox", "create", "--help"],
        vec!["--no-color", "--help"],
        vec!["--json", "--version"],
        vec!["--json", "message", "--help"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_tmpmail"))
            .args(&arguments)
            .output()
            .expect("run tmpmail");
        assert!(output.status.success(), "{arguments:?}");
        assert!(output.stderr.is_empty(), "{arguments:?}");
        assert!(!output.stdout.is_empty(), "{arguments:?}");
    }
}

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
