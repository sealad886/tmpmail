use crate::core::Inbox;
use crate::core::MessageKey;
use crate::error::AppError;
use crate::providers::MessageSummary;
use crate::state::AppState;
use std::time::Duration;

pub fn locally_unseen(state: &AppState, inbox: &str, message: &MessageSummary) -> bool {
    state.is_unseen(inbox, &key(message))
}

pub fn select_inboxes(
    state: &AppState,
    names: &[String],
    all: bool,
) -> Result<Vec<Inbox>, AppError> {
    if all {
        return Ok(state
            .inboxes
            .values()
            .filter(|inbox| inbox.watch_enabled)
            .cloned()
            .collect());
    }
    if names.is_empty() {
        let name = state
            .active_inbox
            .as_deref()
            .ok_or_else(|| AppError::new("inbox_not_found", "no active inbox", 8))?;
        return state
            .inboxes
            .get(name)
            .cloned()
            .map(|inbox| vec![inbox])
            .ok_or_else(|| AppError::new("inbox_not_found", "active inbox not found", 8));
    }
    names
        .iter()
        .map(|name| {
            state.inboxes.get(name).cloned().ok_or_else(|| {
                AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8)
            })
        })
        .collect()
}

pub fn messages_after<'a>(
    messages: &'a [MessageSummary],
    cursor: Option<&MessageKey>,
) -> Vec<&'a MessageSummary> {
    let mut ordered: Vec<_> = messages.iter().collect();
    ordered.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
    cursor.map_or(ordered.clone(), |cursor| {
        ordered
            .into_iter()
            .filter(|message| key(message) > *cursor)
            .collect()
    })
}

pub fn newest_cursor(messages: &[MessageSummary]) -> Option<MessageKey> {
    messages
        .iter()
        .max_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)))
        .map(key)
}

pub fn establish_baseline(
    baselined: &mut std::collections::BTreeSet<String>,
    inbox: &str,
    messages: &[MessageSummary],
) -> Option<MessageKey> {
    if !baselined.insert(inbox.to_owned()) {
        return None;
    }
    newest_cursor(messages)
}

fn key(message: &MessageSummary) -> MessageKey {
    MessageKey {
        created_at: message.created_at.clone(),
        id: message.id.clone(),
    }
}

pub fn retry_delay(failures: u32, floor: Duration, retry_after: Option<Duration>) -> Duration {
    let exponent = failures.min(6);
    let backoff = floor.saturating_mul(1_u32 << exponent);
    retry_after.unwrap_or_default().max(backoff)
}

pub fn retry_delay_with_jitter(
    failures: u32,
    floor: Duration,
    retry_after: Option<Duration>,
    seed: u64,
) -> Duration {
    let base = retry_delay(failures, floor, retry_after);
    let window_ms = (floor.as_millis() / 4) as u64;
    base.saturating_add(Duration::from_millis(if window_ms == 0 {
        0
    } else {
        seed % (window_ms + 1)
    }))
}
