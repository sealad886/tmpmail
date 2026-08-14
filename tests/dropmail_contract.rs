use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tmpmail::providers::{DropMail, DropMailCredentials, ProviderFailure};

fn server(bodies: Vec<&'static str>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for body in bodies {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut bytes = vec![0; 16 * 1024];
            let count = stream.read(&mut bytes).expect("read");
            requests.push(String::from_utf8_lossy(&bytes[..count]).into_owned());
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("respond");
        }
        requests
    });
    (format!("http://{address}/graphql"), handle)
}

fn credentials() -> DropMailCredentials {
    DropMailCredentials {
        token: "af_test-token-current".to_owned(),
        session_id: "session-1".to_owned(),
        address_id: "address-1".to_owned(),
        address: "random@drop.test".to_owned(),
        restore_key: "restore-secret".to_owned(),
        expires_at: "2026-08-14T00:10:00Z".to_owned(),
    }
}

#[test]
fn rejects_legacy_or_short_tokens_before_network_access() {
    let error = DropMail::new("legacy-token", Duration::from_secs(1)).expect_err("legacy token");
    assert_eq!(error.failure, ProviderFailure::Authentication);
    assert!(!error.to_string().contains("legacy-token"));
}

#[test]
fn debug_output_redacts_token_and_restore_key() {
    let rendered = format!("{:?}", credentials());
    assert!(!rendered.contains("af_test-token-current"));
    assert!(!rendered.contains("restore-secret"));
    assert!(rendered.contains("[REDACTED]"));
}

#[tokio::test]
async fn creates_session_and_persists_restore_metadata() {
    let (base, handle) = server(vec![
        r#"{"data":{"introduceSession":{"id":"session-1","expiresAt":"2026-08-14T00:10:00Z","addresses":[{"id":"address-1","address":"random@drop.test","restoreKey":"restore-secret"}]}}}"#,
    ]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    let created = provider.create().await.expect("create");
    assert_eq!(created.address, "random@drop.test");
    assert_eq!(created.restore_key, "restore-secret");
    let requests = handle.join().expect("server");
    assert!(requests[0].starts_with("POST /graphql/af_test-token-current "));
    assert!(requests[0].contains("introduceSession"));
}

#[tokio::test]
async fn parses_session_mail_and_updates_expiry() {
    let (base, handle) = server(vec![
        r#"{"data":{"session":{"id":"session-1","expiresAt":"2026-08-14T00:20:00Z","mails":[{"id":"mail-1","rawSize":123,"fromAddr":"sender@example.test","toAddr":"random@drop.test","receivedAt":"2026-08-14T00:01:00Z","text":"hello","html":"<p>hello</p>","headerSubject":"Subject","downloadUrl":"https://dropmail.me/download/mail/1","attachments":[]}]}}}"#,
    ]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");
    let mut credentials = credentials();

    let messages = provider.messages(&mut credentials).await.expect("messages");
    assert_eq!(messages[0].subject, "Subject");
    assert_eq!(credentials.expires_at, "2026-08-14T00:20:00Z");
    handle.join().expect("server");
}

#[tokio::test]
async fn treats_partial_data_and_errors_as_failure() {
    let (base, handle) = server(vec![
        r#"{"data":{"session":{"id":"session-1","expiresAt":"x","mails":[]}},"errors":[{"message":"quota","extensions":{"code":"RATE_LIMIT_EXCEEDED","captcha":{"id":"never-solve"}}}]}"#,
    ]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    let error = provider
        .messages(&mut credentials())
        .await
        .expect_err("GraphQL errors override data");
    assert!(matches!(error.failure, ProviderFailure::RateLimited { .. }));
    assert!(error.to_string().contains("disabled"));
    assert!(!error.to_string().contains("never-solve"));
    handle.join().expect("server");
}

#[tokio::test]
async fn normalizes_expired_session() {
    let (base, handle) = server(vec![
        r#"{"data":{"session":null},"errors":[{"message":"session_not_found","extensions":{"code":"SESSION_NOT_FOUND"}}]}"#,
    ]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    let error = provider
        .messages(&mut credentials())
        .await
        .expect_err("expired session");
    assert_eq!(error.failure, ProviderFailure::NotFound);
    handle.join().expect("server");
}

#[tokio::test]
async fn rejects_cross_origin_attachment_before_request() {
    let provider = DropMail::with_base_url(
        "http://127.0.0.1:9/graphql",
        "af_test-token-current",
        Duration::from_millis(50),
    )
    .expect("provider");
    let error = provider
        .attachment("http://127.0.0.1.evil.example/file")
        .await
        .expect_err("unsafe origin");
    assert_eq!(error.failure, ProviderFailure::InvalidResponse);
}

#[tokio::test]
async fn lists_only_healthy_api_domains() {
    let (base, handle) = server(vec![
        r#"{"data":{"domains":[{"name":"active.test","availableVia":["API"],"expiresAt":null,"health":"OK"},{"name":"web.test","availableVia":["WEB"],"expiresAt":null,"health":"OK"},{"name":"broken.test","availableVia":["API"],"expiresAt":null,"health":"BROKEN"}]}}"#,
    ]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    assert_eq!(provider.domains().await.expect("domains"), ["active.test"]);
    handle.join().expect("server");
}

#[tokio::test]
async fn uses_schema_correct_delete_address_input() {
    let (base, handle) = server(vec![r#"{"data":{"deleteAddress":true}}"#]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    provider
        .delete_address(&credentials())
        .await
        .expect("delete address");
    let request = &handle.join().expect("server")[0];
    assert!(request.contains("deleteAddress(input:"));
    assert!(request.contains("addressId"));
}
