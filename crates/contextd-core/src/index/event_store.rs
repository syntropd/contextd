//! Append-only disk storage for system events.
//!
//! Manages an indexed JSON Lines event log in `/var/lib/contextd/events.jsonl`.

use crate::error::ContextdError;
use crate::index::event_entry::SystemEvent;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::warn;

/// Manages append-only system event logs on disk.
pub struct EventStore {
    log_path: PathBuf,
    write_lock: Mutex<()>,
}

impl EventStore {
    /// Initializes the event store under the specified root directory.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self, ContextdError> {
        let root_dir = root.as_ref();
        fs::create_dir_all(root_dir)?;
        let log_path = root_dir.join("events.jsonl");

        // Touch file if missing; use create_new to avoid clobbering.
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&log_path)
        {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(ContextdError::Io(e)),
        }

        Ok(Self {
            log_path,
            write_lock: Mutex::new(()),
        })
    }

    /// Appends a new system event to the log file.
    pub fn append(&self, event: &SystemEvent) -> Result<(), ContextdError> {
        let _guard = self.write_lock.lock().map_err(|_| {
            ContextdError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Lock poison"))
        })?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let mut line = serde_json::to_vec(event)
            .map_err(|e| ContextdError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        line.push(b'\n');

        file.write_all(&line)?;
        file.flush()?;
        Ok(())
    }

    /// Queries historical events filtered by unit, timestamp, and maximum result limit.
    ///
    /// TODO(perf): for selective unit filters on large logs, walk the
    /// file backwards from the end so the most recent matching events
    /// surface without scanning the whole log. Current implementation
    /// is O(file size) regardless of selectivity.
    pub fn query(
        &self,
        unit: Option<&str>,
        since_us: u64,
        limit: usize,
    ) -> Result<Vec<SystemEvent>, ContextdError> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_path)?;
        let mut reader = BufReader::with_capacity(32768, file);
        let mut results = Vec::new();
        let mut raw_line = Vec::new();

        loop {
            match reader.read_until(b'\n', &mut raw_line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    return Err(ContextdError::Io(e));
                }
            }

            match std::str::from_utf8(&raw_line) {
                Ok(text) => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        if let Ok(event) = serde_json::from_str::<SystemEvent>(trimmed) {
                            if event.timestamp_us >= since_us {
                                if unit.is_none() || event.unit.as_deref() == unit {
                                    results.push(event);
                                    if results.len() >= limit {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    warn!(
                        "EventStore: skipping non-UTF-8 line of {} bytes",
                        raw_line.len()
                    );
                }
            }
            raw_line.clear();
        }

        Ok(results)
    }
}