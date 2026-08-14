use tempfile::TempDir;
use tmpmail::core::{Capabilities, Inbox, ProviderId};
use tmpmail::state::{AppState, StateStore};

#[test]
fn state_round_trips_and_preserves_active_inbox() {
    let temp = TempDir::new().expect("tempdir");
    let store = StateStore::at(temp.path().join("state.json"));
    let mut state = AppState::default();
    state.insert(Inbox::new(
        "work",
        "user@example.test",
        ProviderId::MailTm,
        "secret:work",
        Capabilities::mailtm(),
    ));
    state.set_active("work").expect("known inbox");

    store.save(&state).expect("state should save");
    let loaded = store.load().expect("state should load");

    assert_eq!(loaded, state);
    assert_eq!(loaded.active_inbox.as_deref(), Some("work"));
}

#[test]
fn corrupt_state_is_reported_instead_of_replaced() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("state.json");
    std::fs::write(&path, b"not json").expect("fixture");

    let error = StateStore::at(path)
        .load()
        .expect_err("corruption must fail");
    assert_eq!(error.code(), "state_corrupt");
}
