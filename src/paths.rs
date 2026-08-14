use crate::error::AppError;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self, AppError> {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| AppError::state("paths", "HOME is not set"))?;
        let config_root = env::var_os("TMPMAIL_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| platform_config_root(&home));
        let state_root = env::var_os("TMPMAIL_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| platform_state_root(&home));
        let cache = env::var_os("TMPMAIL_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| platform_cache_root(&home));
        Ok(Self {
            config: config_root.join("config.toml"),
            state: state_root.join("state.json"),
            cache,
        })
    }
}

#[cfg(target_os = "macos")]
fn platform_config_root(home: &std::path::Path) -> PathBuf {
    home.join("Library/Application Support/tmpmail")
}

#[cfg(not(target_os = "macos"))]
fn platform_config_root(home: &std::path::Path) -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
        .join("tmpmail")
}

#[cfg(target_os = "macos")]
fn platform_state_root(home: &std::path::Path) -> PathBuf {
    platform_config_root(home)
}

#[cfg(not(target_os = "macos"))]
fn platform_state_root(home: &std::path::Path) -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"))
        .join("tmpmail")
}

#[cfg(target_os = "macos")]
fn platform_cache_root(home: &std::path::Path) -> PathBuf {
    home.join("Library/Caches/tmpmail")
}

#[cfg(not(target_os = "macos"))]
fn platform_cache_root(home: &std::path::Path) -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".cache"))
        .join("tmpmail")
}
