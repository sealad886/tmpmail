use tmpmail::config::Config;
use tmpmail::core::ProviderId;

#[test]
fn default_provider_order_is_mailtm_then_dropmail() {
    let config = Config::default();
    assert_eq!(
        config.providers.order,
        vec![ProviderId::MailTm, ProviderId::DropMail]
    );
    assert!(config.providers.disabled.is_empty());
}

#[test]
fn config_serialization_never_contains_secret_fields() {
    let encoded = toml::to_string(&Config::default()).expect("config should serialize");
    for forbidden in ["password", "token", "jwt", "restore_key"] {
        assert!(!encoded.contains(forbidden), "config leaked {forbidden}");
    }
}
