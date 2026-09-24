//! Background configuration drift monitoring task.
//!
//! Inspects tracked configuration files on a periodic interval and records
//! unified diffs whenever drift occurs.

use contextd_core::watcher::{DiffStore, FileTracker};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Background task that continuously scans designated paths for changes.
pub struct WatcherTask {
    tracker: Arc<FileTracker>,
    diff_store: Arc<DiffStore>,
    watched_paths: Vec<PathBuf>,
    poll_interval: Duration,
}

impl WatcherTask {
    /// Constructs a new WatcherTask.
    pub fn new(
        tracker: Arc<FileTracker>,
        diff_store: Arc<DiffStore>,
        watched_paths: Vec<PathBuf>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            tracker,
            diff_store,
            watched_paths,
            poll_interval,
        }
    }

    /// Initializes baselines for existing configuration files without emitting diffs.
    pub fn initialize_baselines(&self) {
        for path in &self.watched_paths {
            if path.is_file() {
                self.record_initial_file(path);
            } else if path.is_dir() {
                self.record_initial_dir(path);
            }
        }
    }

    fn record_initial_file(&self, path: &Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.tracker.set_baseline(&path.to_string_lossy(), &content);
        }
    }

    fn record_initial_dir(&self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    self.record_initial_file(&p);
                }
            }
        }
    }

    /// Runs the periodic scan loop.
    pub async fn run(self) {
        info!("Starting configuration drift watcher loop");
        loop {
            sleep(self.poll_interval).await;
            self.scan_cycle();
        }
    }

    /// Executes a single scan cycle over watched paths.
    pub fn scan_cycle(&self) {
        for path in &self.watched_paths {
            if path.is_file() {
                self.inspect_path(path);
            } else if path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            self.inspect_path(&p);
                        }
                    }
                }
            }
        }
    }

    fn inspect_path(&self, file_path: &Path) {
        match self.tracker.inspect_disk_file(file_path, &self.diff_store) {
            Ok(Some(diff_record)) => {
                info!(
                    "Recorded configuration drift for {}: timestamp {}",
                    diff_record.file_path, diff_record.timestamp_us
                );
            }
            Ok(None) => {
                debug!("No changes for {}", file_path.display());
            }
            Err(e) => {
                warn!("Failed to inspect {}: {}", file_path.display(), e);
            }
        }
    }
}
