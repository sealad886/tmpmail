use serde_json::json;
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
