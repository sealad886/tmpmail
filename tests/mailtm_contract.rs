use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tmpmail::providers::{MailTm, ProviderFailure};

struct Reply {
    status: &'static str,
    content_type: &'static str,
    body: &'static str,
}

fn server(replies: Vec<Reply>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("local address");
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for reply in replies {
            let (mut stream, _) = listener.accept().expect("accept request");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("read timeout");
            let mut bytes = vec![0; 16 * 1024];
            let count = stream.read(&mut bytes).expect("read request");
            let request = String::from_utf8_lossy(&bytes[..count]).into_owned();
            requests.push(request);
            write!(
                stream,
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply.status,
                reply.content_type,
                reply.body.len(),
                reply.body
            )
            .expect("write response");
        }
        requests
    });
    (format!("http://{address}"), handle)
}

#[tokio::test]
async fn discovers_only_active_domains() {
    let (base, handle) = server(vec![Reply {
        status: "200 OK",
        content_type: "application/ld+json",
        body: r#"{"hydra:member":[{"domain":"active.test","isActive":true},{"domain":"retired.test","isActive":false}]}"#,
    }]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");

    assert_eq!(provider.domains().await.expect("domains"), ["active.test"]);
    assert!(handle.join().expect("server")[0].starts_with("GET /domains "));
}

#[tokio::test]
async fn follows_bounded_same_origin_collection_pagination() {
    let (base, handle) = server(vec![
        Reply {
            status: "200 OK",
            content_type: "application/ld+json",
            body: r#"{"hydra:member":[{"domain":"one.test","isActive":true}],"hydra:view":{"hydra:next":"/domains?page=2"}}"#,
        },
        Reply {
            status: "200 OK",
            content_type: "application/ld+json",
            body: r#"{"hydra:member":[{"domain":"two.test","isActive":true}]}"#,
        },
    ]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");

    assert_eq!(
        provider.domains().await.expect("paginated domains"),
        ["one.test", "two.test"]
    );
    let requests = handle.join().expect("server");
    assert!(requests[0].starts_with("GET /domains "));
    assert!(requests[1].starts_with("GET /domains?page=2 "));
}

#[tokio::test]
async fn creates_account_then_authenticates_without_leaking_credentials() {
    let (base, handle) = server(vec![
        Reply {
            status: "201 Created",
            content_type: "application/ld+json",
            body: r#"{"id":"acct-1","address":"user@active.test"}"#,
        },
        Reply {
            status: "200 OK",
            content_type: "application/json",
            body: r#"{"id":"acct-1","token":"jwt-secret"}"#,
        },
    ]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");

    let credentials = provider
        .create("user@active.test", "password-secret")
        .await
        .expect("create");

    assert_eq!(credentials.account_id, "acct-1");
    assert_eq!(credentials.token, "jwt-secret");
    let requests = handle.join().expect("server");
    assert!(requests[0].starts_with("POST /accounts "));
    assert!(requests[1].starts_with("POST /token "));
}

#[tokio::test]
async fn parses_message_list_and_full_message() {
    let (base, handle) = server(vec![
        Reply {
            status: "200 OK",
            content_type: "application/ld+json",
            body: r#"{"hydra:member":[{"id":"m1","from":{"name":"Sender","address":"sender@example.test"},"subject":"Hello","intro":"Preview","seen":false,"hasAttachments":true,"size":42,"createdAt":"2026-08-14T00:00:00Z"}]}"#,
        },
        Reply {
            status: "200 OK",
            content_type: "application/ld+json",
            body: r#"{"id":"m1","from":{"name":"Sender","address":"sender@example.test"},"subject":"Hello","text":"Body","html":["<p>Body</p>"],"seen":false,"attachments":[{"id":"a1","filename":"note.txt","contentType":"text/plain","size":4,"downloadUrl":"/messages/m1/attachment/a1"}],"createdAt":"2026-08-14T00:00:00Z"}"#,
        },
        Reply {
            status: "200 OK",
            content_type: "application/ld+json",
            body: r#"{"id":"m1","downloadUrl":"/sources/m1","data":"From: sender@example.test\r\n\r\nBody"}"#,
        },
    ]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");

    let messages = provider.messages("token").await.expect("messages");
    assert_eq!(messages.len(), 1);
    assert!(!messages[0].seen);
    let message = provider.message("token", "m1").await.expect("message");
    assert_eq!(message.text, "Body");
    assert_eq!(message.attachments[0].filename, "note.txt");
    assert!(
        provider
            .source("token", "m1")
            .await
            .expect("source")
            .starts_with("From: sender@example.test")
    );
    handle.join().expect("server");
}

#[tokio::test]
async fn rejects_malformed_json_and_wrong_content_type() {
    let (base, handle) = server(vec![
        Reply {
            status: "200 OK",
            content_type: "application/json",
            body: "not-json",
        },
        Reply {
            status: "200 OK",
            content_type: "text/html",
            body: "<html>error</html>",
        },
    ]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");

    let malformed = provider.domains().await.expect_err("malformed JSON");
    assert_eq!(malformed.failure, ProviderFailure::InvalidResponse);
    let wrong_type = provider.domains().await.expect_err("wrong content type");
    assert_eq!(wrong_type.failure, ProviderFailure::InvalidResponse);
    handle.join().expect("server");
}

#[tokio::test]
async fn normalizes_rate_limit_without_exposing_response_body() {
    let (base, handle) = server(vec![Reply {
        status: "429 Too Many Requests",
        content_type: "application/json",
        body: r#"{"password":"must-not-escape"}"#,
    }]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");

    let error = provider.domains().await.expect_err("rate limit");
    assert!(matches!(error.failure, ProviderFailure::RateLimited { .. }));
    assert!(!error.to_string().contains("must-not-escape"));
    handle.join().expect("server");
}

#[tokio::test]
async fn rejects_cross_origin_attachment_url_before_request() {
    let provider =
        MailTm::with_base_url("http://127.0.0.1:9", Duration::from_millis(50)).expect("provider");
    let error = provider
        .attachment("token", "http://127.0.0.1.evil.example/secret")
        .await
        .expect_err("cross-origin URL");
    assert_eq!(error.failure, ProviderFailure::InvalidResponse);
}

#[tokio::test]
async fn refreshes_expired_token_once_then_retries() {
    let (base, handle) = server(vec![
        Reply {
            status: "401 Unauthorized",
            content_type: "application/json",
            body: r#"{"message":"expired"}"#,
        },
        Reply {
            status: "200 OK",
            content_type: "application/json",
            body: r#"{"id":"acct-1","token":"fresh-token"}"#,
        },
        Reply {
            status: "200 OK",
            content_type: "application/ld+json",
            body: r#"{"hydra:member":[]}"#,
        },
    ]);
    let provider = MailTm::with_base_url(base, Duration::from_secs(2)).expect("provider");
    let mut credentials = tmpmail::providers::MailTmCredentials {
        account_id: "acct-1".to_owned(),
        address: "user@active.test".to_owned(),
        password: "password-secret".to_owned(),
        token: "expired-token".to_owned(),
    };

    assert!(
        provider
            .messages_with_refresh(&mut credentials)
            .await
            .expect("refreshed list")
            .is_empty()
    );
    assert_eq!(credentials.token, "fresh-token");
    let requests = handle.join().expect("server");
    assert!(requests[0].contains("authorization: Bearer expired-token"));
    assert!(requests[1].starts_with("POST /token "));
    assert!(requests[2].contains("authorization: Bearer fresh-token"));
}

#[test]
fn credential_debug_output_is_redacted() {
    let credentials = tmpmail::providers::MailTmCredentials {
        account_id: "acct-1".to_owned(),
        address: "user@active.test".to_owned(),
        password: "password-secret".to_owned(),
        token: "jwt-secret".to_owned(),
    };
    let rendered = format!("{credentials:?}");
    assert!(!rendered.contains("password-secret"));
    assert!(!rendered.contains("jwt-secret"));
    assert!(rendered.contains("[REDACTED]"));
}
