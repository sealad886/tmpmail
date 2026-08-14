mod mailtm;

pub use mailtm::{MailTm, MailTmCredentials};

use crate::core::ProviderId;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSummary {
    pub id: String,
    pub from: Address,
    pub subject: String,
    pub intro: String,
    pub seen: bool,
    pub has_attachments: bool,
    pub size: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: Address,
    pub subject: String,
    pub text: String,
    pub html: Vec<String>,
    pub seen: bool,
    pub attachments: Vec<Attachment>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedInbox {
    pub account_id: String,
    pub address: String,
    pub secret_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFailure {
    Authentication,
    RateLimited { retry_after_seconds: Option<u64> },
    Unavailable,
    Timeout,
    NotFound,
    Conflict,
    InvalidResponse,
}

#[derive(Debug)]
pub struct ProviderError {
    pub provider: ProviderId,
    pub failure: ProviderFailure,
    pub safe_message: String,
}

impl ProviderError {
    pub fn new(failure: ProviderFailure, message: impl Into<String>) -> Self {
        Self {
            provider: ProviderId::MailTm,
            failure,
            safe_message: message.into(),
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self.failure,
            ProviderFailure::RateLimited { .. }
                | ProviderFailure::Unavailable
                | ProviderFailure::Timeout
        )
    }

    pub fn into_app_error(self) -> AppError {
        let retryable = self.retryable();
        let (code, exit) = match self.failure {
            ProviderFailure::Authentication => ("provider_auth", 4),
            ProviderFailure::RateLimited { .. } => ("provider_rate_limit", 6),
            ProviderFailure::Timeout => ("provider_timeout", 7),
            ProviderFailure::NotFound => ("not_found", 8),
            ProviderFailure::Conflict => ("provider_conflict", 5),
            ProviderFailure::Unavailable | ProviderFailure::InvalidResponse => {
                ("provider_unavailable", 5)
            }
        };
        AppError::new(code, self.safe_message, exit).with_retryable(retryable)
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.safe_message)
    }
}

impl std::error::Error for ProviderError {}
