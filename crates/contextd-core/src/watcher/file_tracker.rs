//! File revision tracker and change detector.
//!
//! Maintains content state and dispatches diff records upon modification.

use crate::error::ContextdError;
use crate::watcher::diff_store::{ConfigDiffRecord, DiffStore};
use crate::watcher::snapshot::compute_text_diff;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

/// Default per-file content cap. Files larger than this are not
/// tracked, to bound memory and prevent accidental snapshotting of
/// large data files (logs, journal dumps) as configuration drift.
pub const DEFAULT_MAX_FILE_BYTES: usize = 1024 * 1024;

/// State tracker for watched configuration files.
pub struct FileTracker {
    snapshots: Mutex<HashMap<String, String>>,
    max_file_bytes: usize,
}

impl FileTracker {
    /// Creates an empty file tracker with the default 1 MiB per-file cap.
    pub fn new() -> Self {
        Self::with_max_file_bytes(DEFAULT_MAX_FILE_BYTES)
    }

    /// Creates a tracker that rejects files larger than `max_file_bytes`.
    pub fn with_max_file_bytes(max_file_bytes: usize) -> Self {
        Self {
            snapshots: Mutex::new(HashMap::new()),
            max_file_bytes,
        }
    }

    /// Sets the baseline content for a watched file without emitting a diff.
    ///
    /// Inputs larger than `max_file_bytes` are silently dropped to bound
    /// memory use. Lock poisoning is logged at `error` level so the
    /// operator notices when another thread panicked holding the
    /// snapshots mutex.
    pub fn set_baseline(&self, path: &str, content: &str) {
        if content.len() > self.max_file_bytes {
            return;
        }
        match self.snapshots.lock() {
            Ok(mut lock) => {
                lock.insert(path.to_string(), content.to_string());
            }
            Err(e) => {
                tracing::error!(
                    "set_baseline: snapshot mutex poisoned for {}: {}",
                    path, e
                );
            }
        }
    }

    /// Checks the current content of a file against its previous revision.
    ///
    /// Inputs larger than `max_file_bytes` are rejected with
    /// `Watcher(...)` so the caller can decide whether to log, alert, or
    /// continue. Returns `Ok(None)` for no diff.
    pub fn check_and_record<P: AsRef<Path>>(
        &self,
        file_path: P,
        current_content: &str,
        diff_store: &DiffStore,
    ) -> Result<Option<ConfigDiffRecord>, ContextdError> {
        if current_content.len() > self.max_file_bytes {
            return Err(ContextdError::Watcher(format!(
                "file exceeds {} byte cap",
                self.max_file_bytes
            )));
        }
        let path_str = file_path.as_ref().to_string_lossy().to_string();
        let mut lock = self.snapshots.lock().map_err(|_| {
            ContextdError::Watcher("Failed to acquire snapshot lock".into())
        })?;

        let old_content = lock.get(&path_str).cloned().unwrap_or_default();
        if let Some(diff) = compute_text_diff(&path_str, &old_content, current_content) {
            let record = diff_store.record_diff(&path_str, &diff)?;
            lock.insert(path_str, current_content.to_string());
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    /// Reads file content from host filesystem and checks for modifications.
    pub fn inspect_disk_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        diff_store: &DiffStore,
    ) -> Result<Option<ConfigDiffRecord>, ContextdError> {
        let p = file_path.as_ref();
        if !p.is_file() {
            return Ok(None);
        }
        let metadata = fs::metadata(p)?;
        if metadata.len() as usize > self.max_file_bytes {
            return Err(ContextdError::Watcher(format!(
                "file {} exceeds {} byte cap",
                p.display(),
                self.max_file_bytes
            )));
        }
        let content = fs::read_to_string(p)?;
        self.check_and_record(p, &content, diff_store)
    }
}