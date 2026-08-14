use crate::core::ProviderId;
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderId>,
    pub fallback_used: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempts: Vec<Attempt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub provider: ProviderId,
    pub duration_ms: u64,
    pub reason: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub schema: &'static str,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub meta: Meta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
}

impl<T> Envelope<T> {
    pub fn success(data: T, meta: Meta) -> Self {
        Self {
            schema: "tmpmail/v1",
            ok: true,
            data: Some(data),
            meta,
            error: None,
        }
    }

    pub fn failure(error: &AppError, meta: Meta) -> Self {
        Self {
            schema: "tmpmail/v1",
            ok: false,
            data: None,
            meta,
            error: Some(ErrorBody {
                code: error.code().to_owned(),
                message: error.to_string(),
                retryable: error.retryable(),
            }),
        }
    }
}
