//! Storage engine for configuration diff snapshots.
//!
//! Stores historical diff records in `/var/lib/contextd/diffs/`.

use crate::error::ContextdError;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A stored configuration file diff snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigDiffRecord {
    /// Host filesystem path that changed (e.g. "/etc/nginx/nginx.conf").
    pub file_path: String,
    /// Unix timestamp in microseconds when the diff was recorded.
    pub timestamp_us: u64,
    /// Unified diff payload.
    pub diff_content: String,
}

/// Diff repository manager on disk.
#[derive(Debug, Clone)]
pub struct DiffStore {
    root_dir: PathBuf,
}

impl DiffStore {
    /// Initializes a new diff store under the designated root path.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, ContextdError> {
        let root_dir = root.as_ref().join("diffs");
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    fn encode_path(file_path: &str) -> String {
        file_path.replace('/', "_")
    }

    /// Records a new configuration diff snapshot to disk.
    pub fn record_diff(&self, file_path: &str, diff_content: &str) -> Result<ConfigDiffRecord, ContextdError> {
        let encoded = Self::encode_path(file_path);
        let path_dir = self.root_dir.join(&encoded);
        fs::create_dir_all(&path_dir)?;

        let now_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let diff_file = path_dir.join(format!("{}.diff", now_us));
        let mut file = File::create(diff_file)?;
        file.write_all(diff_content.as_bytes())?;
        file.flush()?;

        Ok(ConfigDiffRecord {
            file_path: file_path.to_string(),
            timestamp_us: now_us,
            diff_content: diff_content.to_string(),
        })
    }

    /// Lists all diff records recorded for a given file since a microsecond timestamp.
    pub fn list_diffs_for_path(&self, file_path: &str, since_us: u64) -> Result<Vec<ConfigDiffRecord>, ContextdError> {
        let encoded = Self::encode_path(file_path);
        let path_dir = self.root_dir.join(&encoded);
        let mut records = Vec::new();

        if !path_dir.is_dir() {
            return Ok(records);
        }

        for entry in fs::read_dir(path_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(ts_str) = name.strip_suffix(".diff") {
                if let Ok(ts) = ts_str.parse::<u64>() {
                    if ts >= since_us {
                        let mut content = String::new();
                        let mut f = File::open(entry.path())?;
                        f.read_to_string(&mut content)?;
                        records.push(ConfigDiffRecord {
                            file_path: file_path.to_string(),
                            timestamp_us: ts,
                            diff_content: content,
                        });
                    }
                }
            }
        }

        records.sort_by_key(|r| r.timestamp_us);
        Ok(records)
    }

    /// Lists all diff records across all files since a microsecond timestamp.
    pub fn list_recent_diffs(&self, since_us: u64) -> Result<Vec<ConfigDiffRecord>, ContextdError> {
        let mut records = Vec::new();
        if !self.root_dir.is_dir() {
            return Ok(records);
        }

        for dir_entry in fs::read_dir(&self.root_dir)? {
            let dir_entry = dir_entry?;
            if dir_entry.file_type()?.is_dir() {
                for file_entry in fs::read_dir(dir_entry.path())? {
                    let file_entry = file_entry?;
                    let name = file_entry.file_name().to_string_lossy().to_string();
                    if let Some(ts_str) = name.strip_suffix(".diff") {
                        if let Ok(ts) = ts_str.parse::<u64>() {
                            if ts >= since_us {
                                let mut content = String::new();
                                let mut f = File::open(file_entry.path())?;
                                f.read_to_string(&mut content)?;

                                // Extract original file path from unified diff header if present
                                let original_path = content
                                    .lines()
                                    .find(|l| l.starts_with("--- a/"))
                                    .and_then(|l| l.strip_prefix("--- a/"))
                                    .unwrap_or("unknown");

                                records.push(ConfigDiffRecord {
                                    file_path: original_path.to_string(),
                                    timestamp_us: ts,
                                    diff_content: content,
                                });
                            }
                        }
                    }
                }
            }
        }

        records.sort_by_key(|r| r.timestamp_us);
        Ok(records)
    }
}
