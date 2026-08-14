use std::collections::BTreeSet;
use std::time::Duration;
use tmpmail::core::ProviderId;
use tmpmail::providers::{
    ProviderFailure, creation_fallback_eligible, creation_order, enforce_creation_deadline,
    remaining_creation_time,
};

#[test]
fn only_transient_creation_failures_are_fallback_eligible() {
    for failure in [
        ProviderFailure::Unavailable,
        ProviderFailure::Timeout,
        ProviderFailure::RateLimited {
            retry_after_seconds: Some(8),
        },
    ] {
        assert!(creation_fallback_eligible(&failure), "{failure:?}");
    }

    for failure in [
        ProviderFailure::Authentication,
        ProviderFailure::NotFound,
        ProviderFailure::Conflict,
        ProviderFailure::InvalidResponse,
    ] {
        assert!(!creation_fallback_eligible(&failure), "{failure:?}");
    }
}

#[test]
fn explicit_provider_is_pinned_and_preference_respects_exclusions() {
    let preferred = [ProviderId::MailTm, ProviderId::DropMail];
    assert_eq!(
        creation_order(
            Some(ProviderId::MailTm),
            &preferred,
            &[ProviderId::MailTm],
            &BTreeSet::from([ProviderId::MailTm]),
            false,
            false,
        ),
        [ProviderId::MailTm]
    );
    assert_eq!(
        creation_order(
            None,
            &preferred,
            &[ProviderId::MailTm],
            &BTreeSet::new(),
            true,
            false,
        ),
        [ProviderId::DropMail]
    );
}

#[test]
fn custom_address_and_missing_token_exclude_dropmail() {
    let preferred = [ProviderId::DropMail, ProviderId::MailTm];
    assert_eq!(
        creation_order(None, &preferred, &[], &BTreeSet::new(), true, true,),
        [ProviderId::MailTm]
    );
    assert_eq!(
        creation_order(None, &preferred, &[], &BTreeSet::new(), false, false,),
        [ProviderId::MailTm]
    );
}

#[test]
fn all_attempts_share_one_total_deadline() {
    assert_eq!(
        remaining_creation_time(Duration::from_secs(10), Duration::from_secs(7)),
        Some(Duration::from_secs(3))
    );
    assert_eq!(
        remaining_creation_time(Duration::from_secs(10), Duration::from_secs(10)),
        None
    );
    assert_eq!(
        remaining_creation_time(Duration::from_secs(10), Duration::from_secs(12)),
        None
    );
}

#[tokio::test]
async fn multi_step_attempt_cannot_exceed_remaining_deadline() {
    let started = std::time::Instant::now();
    let error = enforce_creation_deadline(ProviderId::MailTm, Duration::from_millis(20), async {
        tokio::time::sleep(Duration::from_millis(15)).await;
        tokio::time::sleep(Duration::from_millis(15)).await;
        Ok::<_, tmpmail::providers::ProviderError>(())
    })
    .await
    .expect_err("deadline");
    assert_eq!(error.failure, ProviderFailure::Timeout);
    assert!(started.elapsed() < Duration::from_millis(100));
}

#[test]
fn normalized_rate_limit_preserves_retry_after_for_watchers() {
    let error = tmpmail::providers::ProviderError::for_provider(
        ProviderId::MailTm,
        ProviderFailure::RateLimited {
            retry_after_seconds: Some(37),
        },
        "rate limited",
    );
    assert_eq!(error.retry_after(), Some(Duration::from_secs(37)));
}
