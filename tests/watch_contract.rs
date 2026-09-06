use std::time::Duration;
use tmpmail::core::{Capabilities, Inbox, MessageKey, ProviderId};
use tmpmail::providers::{Address, MessageSummary};
use tmpmail::state::AppState;
use tmpmail::watch::{
    establish_baseline, locally_unseen, messages_after, newest_cursor, retry_delay,
    retry_delay_with_jitter, select_inboxes,
};

fn message(id: &str, created_at: &str) -> MessageSummary {
    MessageSummary {
        id: id.to_owned(),
        from: Address {
            name: String::new(),
            address: "sender@example.test".to_owned(),
        },
        subject: id.to_owned(),
        intro: String::new(),
        seen: false,
        has_attachments: false,
        size: 1,
        created_at: created_at.to_owned(),
    }
}

#[test]
fn provider_seen_flag_does_not_override_local_unseen_state() {
    let state = AppState::default();
    let mut externally_seen = message("message", "1");
    externally_seen.seen = true;
    assert!(locally_unseen(&state, "inbox", &externally_seen));
}

#[test]
fn baseline_is_established_per_inbox_after_its_first_success() {
    let existing = vec![message("old", "1")];
    let mut baselined = std::collections::BTreeSet::new();

    // Inbox A fails, so no baseline transition occurs. Inbox B succeeds.
    assert_eq!(
        establish_baseline(&mut baselined, "b", &existing)
            .expect("B baseline")
            .id,
        "old"
    );
    assert!(!baselined.contains("a"));

    // A's later first success still establishes a baseline instead of emitting old mail.
    assert_eq!(
        establish_baseline(&mut baselined, "a", &existing)
            .expect("A baseline")
            .id,
        "old"
    );
    assert!(establish_baseline(&mut baselined, "a", &existing).is_none());
}

#[test]
fn cursor_emits_only_later_messages_in_delivery_order() {
    let messages = vec![
        message("three", "3"),
        message("one", "1"),
        message("two", "2"),
    ];
    assert_eq!(
        messages_after(
            &messages,
            Some(&MessageKey {
                created_at: "1".to_owned(),
                id: "one".to_owned()
            }),
        )
        .iter()
        .map(|m| m.id.as_str())
        .collect::<Vec<_>>(),
        ["two", "three"]
    );
    assert_eq!(
        newest_cursor(&messages).map(|key| key.id),
        Some("three".to_owned())
    );
}

#[test]
fn missing_cursor_uses_ordered_high_water_without_replay() {
    let messages = vec![message("four", "4"), message("five", "5")];
    let old = MessageKey {
        created_at: "3".to_owned(),
        id: "three".to_owned(),
    };
    assert_eq!(messages_after(&messages, Some(&old)).len(), 2);
    let new = MessageKey {
        created_at: "6".to_owned(),
        id: "six".to_owned(),
    };
    assert!(messages_after(&messages, Some(&new)).is_empty());
}

#[test]
fn backoff_respects_provider_floor_and_retry_after() {
    assert_eq!(
        retry_delay(2, Duration::from_secs(8), None),
        Duration::from_secs(32)
    );
    assert_eq!(
        retry_delay(1, Duration::from_secs(8), Some(Duration::from_secs(60))),
        Duration::from_secs(60)
    );
    let jittered = retry_delay_with_jitter(1, Duration::from_secs(8), None, 7);
    assert!(jittered >= Duration::from_secs(16));
    assert!(jittered <= Duration::from_secs(18));
}

#[test]
fn all_uses_opt_in_while_named_selection_is_temporary() {
    let mut state = AppState::default();
    let mut enabled = Inbox::new(
        "enabled",
        "e@example.test",
        ProviderId::MailTm,
        "secret:e",
        Capabilities::mailtm(),
    );
    enabled.watch_enabled = true;
    state.insert(enabled);
    state.insert(Inbox::new(
        "manual",
        "m@example.test",
        ProviderId::MailTm,
        "secret:m",
        Capabilities::mailtm(),
    ));
    assert_eq!(
        select_inboxes(&state, &[], true)
            .expect("all")
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>(),
        ["enabled"]
    );
    assert_eq!(
        select_inboxes(&state, &["manual".to_owned()], false).expect("named")[0].name,
        "manual"
    );
}
