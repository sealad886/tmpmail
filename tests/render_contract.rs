use serde_json::json;
use tmpmail::error::AppError;
use tmpmail::providers::Attachment;
use tmpmail::render::{Envelope, Meta};

#[test]
fn success_envelope_has_stable_v1_shape() {
    let envelope = Envelope::success(json!({"address": "user@example.test"}), Meta::default());
    let value = serde_json::to_value(envelope).expect("serialize");

    assert_eq!(value["schema"], "tmpmail/v1");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["address"], "user@example.test");
    assert_eq!(value["meta"]["fallback_used"], false);
    assert!(value.get("error").is_none());
}

#[test]
fn failure_envelope_has_stable_v1_shape() {
    let error = AppError::new("provider_timeout", "provider timed out", 7).with_retryable(true);
    let value = serde_json::to_value(Envelope::<serde_json::Value>::failure(
        &error,
        Meta::default(),
    ))
    .expect("serialize");
    assert_eq!(value["schema"], "tmpmail/v1");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "provider_timeout");
    assert_eq!(value["error"]["retryable"], true);
    assert!(value.get("data").is_none());
}

#[test]
fn public_attachment_json_excludes_provider_download_url() {
    let attachment = Attachment {
        id: "a".to_owned(),
        filename: "safe.txt".to_owned(),
        content_type: "text/plain".to_owned(),
        size: 1,
        download_url: "https://provider.test/private?token=secret".to_owned(),
    };
    let encoded = serde_json::to_string(&attachment).expect("serialize");
    assert!(!encoded.contains("provider.test"));
    assert!(!encoded.contains("download_url"));
}
