use crate::core::ProviderId;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub providers: ProviderConfig,
    pub timeout_seconds: u64,
    pub color: bool,
    pub browser: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            providers: ProviderConfig::default(),
            timeout_seconds: 15,
            color: true,
            browser: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    pub order: Vec<ProviderId>,
    pub disabled: BTreeSet<ProviderId>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            order: vec![ProviderId::MailTm, ProviderId::DropMail],
            disabled: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<Config, AppError> {
        if !self.path.exists() {
            return Ok(Config::default());
        }
        let raw = fs::read_to_string(&self.path)
            .map_err(|error| AppError::state("config_read", error.to_string()))?;
        toml::from_str(&raw)
            .map_err(|error| AppError::state("config_invalid", format!("invalid config: {error}")))
    }

    pub fn save(&self, config: &Config) -> Result<(), AppError> {
        let encoded = toml::to_string_pretty(config)
            .map_err(|error| AppError::state("config_encode", error.to_string()))?;
        atomic_write(&self.path, encoded.as_bytes())
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| AppError::state("config_write", error.to_string()))?;
    }
    let temporary = path.with_extension("toml.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| AppError::state("config_write", error.to_string()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| AppError::state("config_write", error.to_string()))?;
    fs::rename(&temporary, path).map_err(|error| AppError::state("config_write", error.to_string()))
}
