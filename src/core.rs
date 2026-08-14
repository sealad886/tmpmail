use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderId {
    MailTm,
    DropMail,
}

impl ProviderId {
    pub const ALL: [Self; 2] = [Self::MailTm, Self::DropMail];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MailTm => "mailtm",
            Self::DropMail => "dropmail",
        }
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "mailtm" | "mail.tm" => Ok(Self::MailTm),
            "dropmail" | "dropmail.me" => Ok(Self::DropMail),
            _ => Err(format!("unknown provider: {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    pub custom_address: bool,
    pub domain_discovery: bool,
    pub remote_delete: bool,
    pub attachments: bool,
    pub recoverable: bool,
    pub minimum_poll_seconds: u64,
}

impl Capabilities {
    pub fn mailtm() -> Self {
        Self {
            custom_address: true,
            domain_discovery: true,
            remote_delete: true,
            attachments: true,
            recoverable: true,
            minimum_poll_seconds: 8,
        }
    }

    pub fn dropmail() -> Self {
        Self {
            custom_address: false,
            domain_discovery: false,
            remote_delete: false,
            attachments: true,
            recoverable: true,
            minimum_poll_seconds: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inbox {
    pub name: String,
    pub address: String,
    pub provider: ProviderId,
    pub secret_ref: String,
    pub capabilities: Capabilities,
    #[serde(default)]
    pub watch_enabled: bool,
}

impl Inbox {
    pub fn new(
        name: impl Into<String>,
        address: impl Into<String>,
        provider: ProviderId,
        secret_ref: impl Into<String>,
        capabilities: Capabilities,
    ) -> Self {
        Self {
            name: name.into(),
            address: address.into(),
            provider,
            secret_ref: secret_ref.into(),
            capabilities,
            watch_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acknowledgements {
    #[serde(default)]
    pub high_water: u64,
    #[serde(default)]
    pub gaps: BTreeSet<u64>,
}

impl Acknowledgements {
    pub fn acknowledge(&mut self, sequence: u64) {
        if sequence <= self.high_water {
            self.gaps.remove(&sequence);
            return;
        }
        for skipped in (self.high_water + 1)..sequence {
            self.gaps.insert(skipped);
        }
        self.high_water = sequence;
    }

    pub fn is_unseen(&self, sequence: u64) -> bool {
        sequence > self.high_water || self.gaps.contains(&sequence)
    }
}
