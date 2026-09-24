//! Background configuration drift monitoring task.
//!
//! Inspects tracked configuration files on a periodic interval and records
//! unified diffs whenever drift occurs.

use contextd_core::watcher::{DiffStore, FileTracker};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Maximum recursion depth for `walk_dir`. Caps stack use even if a
/// misconfigured watched path contains a long chain of nested
/// directories.
const MAX_WALK_DEPTH: usize = 32;

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
                walk_dir(path, &mut |p| self.record_initial_file(p));
            }
        }
    }

    fn record_initial_file(&self, path: &Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            self.tracker.set_baseline(&path.to_string_lossy(), &content);
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
                walk_dir(path, &mut |p| self.inspect_path(p));
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

/// Walks a directory recursively, invoking `visit` for every regular file.
///
/// Skips symlinks entirely (does not follow them), tracks visited
/// `(st_dev, st_ino)` pairs to break hardlink cycles, and caps depth
/// at `MAX_WALK_DEPTH` so a misconfigured watched path cannot blow the
/// stack. Public so the QA edge tests can exercise the walker
/// directly without going through a live WatcherTask.
pub fn walk_dir<F: FnMut(&Path)>(dir: &Path, visit: &mut F) {
    let mut visited: Vec<(u64, u64)> = Vec::new();
    walk_dir_inner(dir, visit, &mut visited, 0);
}

fn walk_dir_inner<F: FnMut(&Path)>(
    dir: &Path,
    visit: &mut F,
    visited: &mut Vec<(u64, u64)>,
    depth: usize,
) {
    if depth >= MAX_WALK_DEPTH {
        warn!(
            "walk_dir: max depth {} reached at {}; stopping descent",
            MAX_WALK_DEPTH,
            dir.display()
        );
        return;
    }
    if let Ok(md) = std::fs::metadata(dir) {
        let key = (md.dev(), md.ino());
        if visited.contains(&key) {
            return;
        }
        visited.push(key);
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk_dir_inner(&p, visit, visited, depth + 1);
        } else if ft.is_file() {
            visit(&p);
        }
        // Symlinks (ft.is_symlink()) are skipped entirely because
        // neither is_dir() nor is_file() returns true for a symlink
        // (DirEntry::file_type reports the symlink's own type, not
        // the target's). This is what prevents cycles.
    }
}
