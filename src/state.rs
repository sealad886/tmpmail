use crate::core::{Acknowledgements, Inbox};
use crate::error::AppError;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppState {
    pub schema_version: u32,
    pub active_inbox: Option<String>,
    #[serde(default)]
    pub inboxes: BTreeMap<String, Inbox>,
    #[serde(default)]
    pub acknowledgements: BTreeMap<String, Acknowledgements>,
    #[serde(default)]
    pub acknowledged_ids: BTreeMap<String, BTreeSet<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            active_inbox: None,
            inboxes: BTreeMap::new(),
            acknowledgements: BTreeMap::new(),
            acknowledged_ids: BTreeMap::new(),
        }
    }
}

impl AppState {
    pub fn insert(&mut self, inbox: Inbox) {
        self.inboxes.insert(inbox.name.clone(), inbox);
    }

    pub fn set_active(&mut self, name: &str) -> Result<(), AppError> {
        if !self.inboxes.contains_key(name) {
            return Err(AppError::new(
                "inbox_not_found",
                format!("inbox not found: {name}"),
                8,
            ));
        }
        self.active_inbox = Some(name.to_owned());
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<Inbox, AppError> {
        let inbox = self.inboxes.remove(name).ok_or_else(|| {
            AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8)
        })?;
        self.acknowledgements.remove(name);
        self.acknowledged_ids.remove(name);
        if self.active_inbox.as_deref() == Some(name) {
            self.active_inbox = self.inboxes.keys().next().cloned();
        }
        Ok(inbox)
    }

    pub fn acknowledge_id(&mut self, inbox: &str, message: &str) {
        self.acknowledged_ids
            .entry(inbox.to_owned())
            .or_default()
            .insert(message.to_owned());
    }

    pub fn is_acknowledged(&self, inbox: &str, message: &str) -> bool {
        self.acknowledged_ids
            .get(inbox)
            .is_some_and(|ids| ids.contains(message))
    }
}

#[derive(Debug, Clone)]
pub struct StateStore {
    path: PathBuf,
}

impl StateStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<AppState, AppError> {
        if !self.path.exists() {
            return Ok(AppState::default());
        }
        let bytes = fs::read(&self.path)
            .map_err(|error| AppError::state("state_read", error.to_string()))?;
        let state: AppState = serde_json::from_slice(&bytes).map_err(|error| {
            AppError::state("state_corrupt", format!("state file is invalid: {error}"))
        })?;
        if state.schema_version != 1 {
            return Err(AppError::state(
                "state_schema",
                format!("unsupported state schema: {}", state.schema_version),
            ));
        }
        Ok(state)
    }

    pub fn save(&self, state: &AppState) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| AppError::state("state_write", error.to_string()))?;
        }
        let lock_path = self.path.with_extension("lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|error| AppError::state("state_lock", error.to_string()))?;
        lock.lock_exclusive()
            .map_err(|error| AppError::state("state_lock", error.to_string()))?;
        let result = self.save_locked(state);
        let _ = FileExt::unlock(&lock);
        result
    }

    fn save_locked(&self, state: &AppState) -> Result<(), AppError> {
        let bytes = serde_json::to_vec(state)
            .map_err(|error| AppError::state("state_encode", error.to_string()))?;
        let temporary = self.path.with_extension("json.tmp");
        let mut file = File::create(&temporary)
            .map_err(|error| AppError::state("state_write", error.to_string()))?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| AppError::state("state_write", error.to_string()))?;
        fs::rename(&temporary, &self.path)
            .map_err(|error| AppError::state("state_write", error.to_string()))
    }
}
