use tempfile::TempDir;
use tmpmail::core::{Acknowledgements, Capabilities, Inbox, MessageKey, ProviderId};
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
fn conditional_remove_preserves_a_same_name_replacement() {
    let mut state = AppState::default();
    state.insert(Inbox::new(
        "same",
        "replacement@example.test",
        ProviderId::MailTm,
        "secret:new",
        Capabilities::mailtm(),
    ));

    let error = state
        .remove_matching("same", ProviderId::MailTm, "secret:old")
        .expect_err("replacement identity must not be removed");

    assert_eq!(error.code(), "state_conflict");
    assert_eq!(error.exit_code(), 10);
    assert_eq!(state.inboxes["same"].secret_ref, "secret:new");
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

fn key(created_at: &str, id: &str) -> MessageKey {
    MessageKey {
        created_at: created_at.to_owned(),
        id: id.to_owned(),
    }
}

#[test]
fn acknowledgement_high_water_preserves_skipped_older_messages() {
    let mut acknowledgements = Acknowledgements::default();
    let visible = vec![key("1", "one"), key("2", "two"), key("3", "three")];
    acknowledgements.acknowledge(key("3", "three"), visible.clone());

    assert!(!acknowledgements.is_unseen(&key("3", "three")));
    assert!(acknowledgements.is_unseen(&key("1", "one")));
    assert!(acknowledgements.is_unseen(&key("2", "two")));

    acknowledgements.acknowledge(key("1", "one"), visible);
    assert!(!acknowledgements.is_unseen(&key("1", "one")));
    assert!(acknowledgements.is_unseen(&key("2", "two")));
}

#[test]
fn acknowledgement_gap_set_is_bounded_to_current_provider_visibility() {
    let mut acknowledgements = tmpmail::core::Acknowledgements::default();
    for page in 1..=200 {
        let older = MessageKey {
            created_at: format!("{page:04}-a"),
            id: format!("older-{page}"),
        };
        let newer = MessageKey {
            created_at: format!("{page:04}-b"),
            id: format!("newer-{page}"),
        };
        acknowledgements.acknowledge(newer.clone(), [older, newer]);
        assert!(acknowledgements.gaps.len() <= 1);
    }
}

#[test]
fn schema_one_migrates_with_backup_and_legacy_acknowledgements() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("state.json");
    std::fs::write(&path, br#"{"schema_version":1,"active_inbox":null,"inboxes":{},"acknowledgements":{},"acknowledged_ids":{"work":["old"]}}"#).expect("fixture");

    let state = StateStore::at(&path).load().expect("migration succeeds");
    assert_eq!(state.schema_version, 2);
    assert!(!state.is_unseen("work", &key("1", "old")));
    assert!(path.with_extension("json.v1.bak").exists());
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).expect("migrated state")).expect("json");
    assert_eq!(persisted["schema_version"], 2);
}

#[test]
fn schema_one_preserves_nonempty_sequence_acknowledgements() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("state.json");
    std::fs::write(&path, br#"{"schema_version":1,"active_inbox":null,"inboxes":{},"acknowledgements":{"work":{"high_water":3,"gaps":[2]}},"acknowledged_ids":{}}"#).expect("fixture");
    let state = StateStore::at(path).load().expect("migration");
    assert!(!state.is_unseen("work", &key("x", "1")));
    assert!(state.is_unseen("work", &key("x", "2")));
    assert!(!state.is_unseen("work", &key("x", "3")));
}

#[test]
fn concurrent_updates_do_not_lose_inbox_mutations() {
    let temp = TempDir::new().expect("tempdir");
    let path = std::sync::Arc::new(temp.path().join("state.json"));
    let mut workers = Vec::new();
    for index in 0..16 {
        let path = path.clone();
        workers.push(std::thread::spawn(move || {
            StateStore::at(path.as_ref())
                .update(|state| {
                    let name = format!("inbox-{index}");
                    state.insert(Inbox::new(
                        &name,
                        format!("{name}@example.test"),
                        ProviderId::MailTm,
                        format!("secret:{name}"),
                        Capabilities::mailtm(),
                    ));
                    Ok(())
                })
                .expect("locked update");
        }));
    }
    for worker in workers {
        worker.join().expect("worker");
    }
    assert_eq!(
        StateStore::at(path.as_ref())
            .load()
            .expect("load")
            .inboxes
            .len(),
        16
    );
}

#[test]
fn migration_and_concurrent_updates_share_one_locked_transaction() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("state.json");
    std::fs::write(
        &path,
        br#"{"schema_version":1,"active_inbox":null,"inboxes":{},"acknowledgements":{}}"#,
    )
    .expect("legacy fixture");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let mut threads = Vec::new();
    for name in ["one", "two"] {
        let path = path.clone();
        let barrier = barrier.clone();
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            StateStore::at(path)
                .update(|state| {
                    state.insert(Inbox::new(
                        name,
                        format!("{name}@example.test"),
                        ProviderId::MailTm,
                        format!("secret:{name}"),
                        Capabilities::mailtm(),
                    ));
                    Ok(())
                })
                .expect("locked update");
        }));
    }
    barrier.wait();
    for thread in threads {
        thread.join().expect("thread");
    }
    let state = StateStore::at(&path).load().expect("migrated state");
    assert_eq!(state.schema_version, 2);
    assert!(state.inboxes.contains_key("one"));
    assert!(state.inboxes.contains_key("two"));
}

#[test]
fn update_refuses_newer_schema_without_rewriting_it() {
    let temp = TempDir::new().expect("tempdir");
    let path = temp.path().join("state.json");
    let original = br#"{"schema_version":99,"active_inbox":null,"inboxes":{}}"#;
    std::fs::write(&path, original).expect("fixture");
    let error = StateStore::at(&path)
        .update(|_| Ok(()))
        .expect_err("new schema refused");
    assert_eq!(error.code(), "state_schema");
    assert_eq!(std::fs::read(path).expect("unchanged"), original);
}
