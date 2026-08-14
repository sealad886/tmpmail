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

pub fn terminal_text(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .chars()
        .filter(|character| {
            matches!(character, '\n' | '\t')
                || (!character.is_control() && !is_bidi_control(*character))
        })
        .collect()
}

pub fn html_to_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut inside_tag = false;
    let mut tag = String::new();
    for character in value.chars() {
        match character {
            '<' if !inside_tag => {
                inside_tag = true;
                tag.clear();
            }
            '>' if inside_tag => {
                inside_tag = false;
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if matches!(
                    name.as_str(),
                    "br" | "p" | "div" | "li" | "tr" | "h1" | "h2" | "h3"
                ) && !output.ends_with('\n')
                {
                    output.push('\n');
                }
            }
            value if inside_tag => tag.push(value),
            value => output.push(value),
        }
    }
    terminal_text(
        &output
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'"),
    )
}

pub fn safe_html_document(plain_text: &str) -> String {
    let escaped = plain_text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    format!(
        "<!doctype html><meta charset=\"utf-8\"><meta name=\"referrer\" content=\"no-referrer\"><title>tmpmail message</title><style>body{{font:16px/1.55 system-ui;max-width:72ch;margin:3rem auto;padding:0 1rem}}pre{{white-space:pre-wrap;overflow-wrap:anywhere}}</style><pre>{escaped}</pre>"
    )
}

fn is_bidi_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
    )
}
