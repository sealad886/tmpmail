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

fn raw_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("address");
    let download_url = format!("http://{address}/download/mail-1");
    let session = format!(
        r#"{{"data":{{"session":{{"id":"session-1","expiresAt":"2026-08-14T00:20:00Z","mails":[{{"id":"mail-1","rawSize":5,"fromAddr":"sender@example.test","toAddr":"random@drop.test","receivedAt":"2026-08-14T00:01:00Z","text":"hello","html":null,"headerSubject":"Subject","downloadUrl":"{download_url}","attachments":[]}}]}}}}}}"#
    );
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for (content_type, body) in [
            ("application/json", session),
            ("message/rfc822", "hello".to_owned()),
        ] {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut bytes = vec![0; 16 * 1024];
            let count = stream.read(&mut bytes).expect("read");
            requests.push(String::from_utf8_lossy(&bytes[..count]).into_owned());
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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

#[test]
fn serialized_inbox_credentials_do_not_duplicate_provider_token() {
    let encoded = serde_json::to_string(&credentials()).expect("serialize");
    assert!(!encoded.contains("af_test-token-current"));
    assert!(!encoded.contains("\"token\""));
}

#[test]
fn saved_inbox_credentials_bind_the_current_provider_token() {
    let encoded = serde_json::to_vec(&credentials()).expect("serialize");
    let mut restored: DropMailCredentials = serde_json::from_slice(&encoded).expect("restore");
    assert!(restored.token.is_empty());
    restored.bind_token("af_rotated-token-current");
    assert_eq!(restored.token, "af_rotated-token-current");
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

    let message = provider
        .message(&mut credentials, "mail-1")
        .await
        .expect("message");
    assert_eq!(message.subject, "Subject");
    assert!(message.raw.is_empty());
    assert_eq!(credentials.expires_at, "2026-08-14T00:20:00Z");
    let requests = handle.join().expect("server");
    assert!(!requests[0].contains(" raw "));
}

#[tokio::test]
async fn fetches_raw_source_only_for_explicit_raw_reads() {
    let (base, handle) = raw_server();
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    let (message, response, size) = provider
        .raw_response(&mut credentials(), "mail-1")
        .await
        .expect("raw message");
    assert!(message.raw.is_empty());
    assert_eq!(size, 5);
    assert_eq!(response.text().await.expect("raw body"), "hello");
    let requests = handle.join().expect("server");
    assert!(!requests[0].contains(" raw "));
    assert!(requests[0].contains("downloadUrl"));
    assert!(requests[1].starts_with("GET /download/mail-1 "));
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
async fn restores_expired_session_then_retries() {
    let (base, handle) = server(vec![
        r#"{"data":{"session":null},"errors":[{"message":"session_not_found","extensions":{"code":"SESSION_NOT_FOUND"}}]}"#,
        r#"{"data":{"introduceSession":{"id":"session-2","expiresAt":"2026-08-14T00:30:00Z","addresses":[{"id":"temporary","address":"temporary@drop.test","restoreKey":"temporary-key"}]}}}"#,
        r#"{"data":{"restoreAddress":{"id":"restored-address"}}}"#,
        r#"{"data":{"session":{"id":"session-2","expiresAt":"2026-08-14T00:30:00Z","mails":[]}}}"#,
    ]);
    let provider = DropMail::with_base_url(base, "af_test-token-current", Duration::from_secs(2))
        .expect("provider");

    let mut credentials = credentials();
    assert!(
        provider
            .messages(&mut credentials)
            .await
            .expect("restored")
            .is_empty()
    );
    assert_eq!(credentials.session_id, "session-2");
    assert_eq!(credentials.address_id, "restored-address");
    let requests = handle.join().expect("server");
    assert!(requests[1].contains("introduceSession"));
    assert!(requests[2].contains("restoreAddress"));
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
