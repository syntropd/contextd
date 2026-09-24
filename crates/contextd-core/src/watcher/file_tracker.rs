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

/// State tracker for watched configuration files.
pub struct FileTracker {
    snapshots: Mutex<HashMap<String, String>>,
}

impl FileTracker {
    /// Creates an empty file tracker.
    pub fn new() -> Self {
        Self {
            snapshots: Mutex::new(HashMap::new()),
        }
    }

    /// Sets the baseline content for a watched file without emitting a diff.
    pub fn set_baseline(&self, path: &str, content: &str) {
        if let Ok(mut lock) = self.snapshots.lock() {
            lock.insert(path.to_string(), content.to_string());
        }
    }

    /// Checks the current content of a file against its previous revision.
    ///
    /// If changes exist, generates a unified diff, commits it to the DiffStore,
    /// and updates the baseline snapshot.
    pub fn check_and_record<P: AsRef<Path>>(
        &self,
        file_path: P,
        current_content: &str,
        diff_store: &DiffStore,
    ) -> Result<Option<ConfigDiffRecord>, ContextdError> {
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
        let content = fs::read_to_string(p)?;
        self.check_and_record(p, &content, diff_store)
    }
}
