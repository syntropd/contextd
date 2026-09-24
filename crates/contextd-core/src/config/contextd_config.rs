//! Contextd configuration parser and default values.
//!
//! Loads settings from `/etc/syntrop/contextd.conf`.

use crate::error::ContextdError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Default configuration path for contextd.
pub const DEFAULT_CONFIG_PATH: &str = "/etc/syntrop/contextd.conf";

/// Default storage root for diffs and event history.
pub const DEFAULT_STORAGE_PATH: &str = "/var/lib/contextd";

/// Default Varlink Unix domain socket path.
pub const DEFAULT_SOCKET_PATH: &str = "/run/syntrop/io.syntrop.Context1";

/// Daemon configuration options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextdConfig {
    /// Root path for historical diffs and event logs.
    pub storage_path: PathBuf,
    /// Varlink Unix domain socket path.
    pub socket_path: PathBuf,
    /// Directories monitored for configuration drift.
    pub watched_directories: Vec<PathBuf>,
    /// Package manager transaction log paths.
    pub package_log_paths: Vec<PathBuf>,
    /// Days to retain historical diffs and events before pruning.
    pub retention_days: u32,
}

impl Default for ContextdConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from(DEFAULT_STORAGE_PATH),
            socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
            watched_directories: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/usr/lib/systemd/system"),
            ],
            package_log_paths: vec![
                PathBuf::from("/var/log/dnf.log"),
                PathBuf::from("/var/log/dpkg.log"),
                PathBuf::from("/var/log/pacman.log"),
            ],
            retention_days: 30,
        }
    }
}

impl ContextdConfig {
    /// Loads configuration from a specified filesystem path, falling back to defaults.
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self, ContextdError> {
        let p = path.as_ref();
        if !p.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(p)?;
        toml::from_str(&content).map_err(|e| ContextdError::Config(e.to_string()))
    }
}
