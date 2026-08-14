use crate::core::{Acknowledgements, Inbox, MessageKey};
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
    #[serde(default)]
    pub watch_cursors: BTreeMap<String, MessageKey>,
    #[serde(default)]
    pub watch_baselined: BTreeSet<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            schema_version: 2,
            active_inbox: None,
            inboxes: BTreeMap::new(),
            acknowledgements: BTreeMap::new(),
            acknowledged_ids: BTreeMap::new(),
            watch_cursors: BTreeMap::new(),
            watch_baselined: BTreeSet::new(),
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
        self.watch_cursors.remove(name);
        self.watch_baselined.remove(name);
        if self.active_inbox.as_deref() == Some(name) {
            self.active_inbox = self.inboxes.keys().next().cloned();
        }
        Ok(inbox)
    }

    pub fn acknowledge(
        &mut self,
        inbox: &str,
        key: MessageKey,
        visible: impl IntoIterator<Item = MessageKey>,
    ) {
        self.acknowledgements
            .entry(inbox.to_owned())
            .or_default()
            .acknowledge(key.clone(), visible);
        if let Some(legacy) = self.acknowledged_ids.get_mut(inbox) {
            legacy.remove(&key.id);
            if legacy.is_empty() {
                self.acknowledged_ids.remove(inbox);
            }
        }
    }

    pub fn is_unseen(&self, inbox: &str, key: &MessageKey) -> bool {
        if self
            .acknowledged_ids
            .get(inbox)
            .is_some_and(|ids| ids.contains(&key.id))
        {
            return false;
        }
        self.acknowledgements
            .get(inbox)
            .is_none_or(|state| state.is_unseen(key))
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
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
            AppError::state("state_corrupt", format!("state file is invalid: {error}"))
        })?;
        let version = value
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1);
        if version > 2 {
            return Err(AppError::state(
                "state_schema",
                format!("unsupported state schema: {version}"),
            ));
        }
        if version == 1 {
            let backup = self.path.with_extension("json.v1.bak");
            fs::copy(&self.path, &backup)
                .map_err(|error| AppError::state("state_migration", error.to_string()))?;
            let object = value.as_object_mut().ok_or_else(|| {
                AppError::state("state_corrupt", "state file root must be an object")
            })?;
            migrate_v1_acknowledgements(object)?;
            object.insert("schema_version".to_owned(), serde_json::Value::from(2));
        }
        let state: AppState = serde_json::from_value(value).map_err(|error| {
            AppError::state("state_corrupt", format!("state file is invalid: {error}"))
        })?;
        if version == 1
            && let Err(error) = self.save(&state)
        {
            let backup = self.path.with_extension("json.v1.bak");
            let _ = fs::copy(backup, &self.path);
            return Err(error);
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

    pub fn update<T>(
        &self,
        mutation: impl FnOnce(&mut AppState) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let _ = self.load()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| AppError::state("state_write", error.to_string()))?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.path.with_extension("lock"))
            .map_err(|error| AppError::state("state_lock", error.to_string()))?;
        lock.lock_exclusive()
            .map_err(|error| AppError::state("state_lock", error.to_string()))?;
        let mut state = if self.path.exists() {
            let bytes = fs::read(&self.path)
                .map_err(|error| AppError::state("state_read", error.to_string()))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
                AppError::state("state_corrupt", format!("state file is invalid: {error}"))
            })?;
            let version = value
                .get("schema_version")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1);
            if version != 2 {
                return Err(AppError::state(
                    "state_schema",
                    format!("unsupported state schema: {version}"),
                ));
            }
            serde_json::from_value(value).map_err(|error| {
                AppError::state("state_corrupt", format!("state file is invalid: {error}"))
            })?
        } else {
            AppState::default()
        };
        let value = mutation(&mut state)?;
        let result = self.save_locked(&state);
        let _ = FileExt::unlock(&lock);
        result.map(|()| value)
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
            .map_err(|error| AppError::state("state_write", error.to_string()))?;
        if let Some(parent) = self.path.parent()
            && let Ok(directory) = File::open(parent)
        {
            directory
                .sync_all()
                .map_err(|error| AppError::state("state_write", error.to_string()))?;
        }
        Ok(())
    }
}

fn migrate_v1_acknowledgements(
    object: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<(), AppError> {
    let old = object.remove("acknowledgements").unwrap_or_default();
    let acknowledged = object
        .entry("acknowledged_ids".to_owned())
        .or_insert_with(|| serde_json::json!({}));
    let acknowledged = acknowledged.as_object_mut().ok_or_else(|| {
        AppError::state("state_migration", "legacy acknowledgement IDs are invalid")
    })?;
    let Some(inboxes) = old.as_object() else {
        return Ok(());
    };
    for (inbox, value) in inboxes {
        let high_water = value
            .get("high_water")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if high_water > 100_000 {
            return Err(AppError::state(
                "state_migration",
                "legacy acknowledgement range is too large",
            ));
        }
        let gaps: BTreeSet<u64> = value
            .get("gaps")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_u64)
            .collect();
        let ids = acknowledged
            .entry(inbox.clone())
            .or_insert_with(|| serde_json::json!([]));
        let ids = ids.as_array_mut().ok_or_else(|| {
            AppError::state(
                "state_migration",
                "legacy inbox acknowledgements are invalid",
            )
        })?;
        ids.extend(
            (1..=high_water)
                .filter(|id| !gaps.contains(id))
                .map(|id| serde_json::Value::String(id.to_string())),
        );
    }
    Ok(())
}
