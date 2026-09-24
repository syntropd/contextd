//! Schema definition for historical system events.

use serde::{Deserialize, Serialize};

/// Structured record of a historical system event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Unique event identifier.
    pub id: String,
    /// Event occurrence timestamp in microseconds.
    pub timestamp_us: u64,
    /// Subsystem or component that emitted the event.
    pub source: String,
    /// Associated systemd unit name, if applicable.
    pub unit: Option<String>,
    /// Brief human-readable summary of the event.
    pub summary: String,
    /// Detailed diagnostic payload or diff snippet.
    pub details: Option<String>,
}

impl SystemEvent {
    /// Constructs a new system event with generated timestamp.
    pub fn new(source: &str, unit: Option<&str>, summary: &str, details: Option<&str>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        Self {
            id: format!("evt-{}-{}", ts, source),
            timestamp_us: ts,
            source: source.to_string(),
            unit: unit.map(ToString::to_string),
            summary: summary.to_string(),
            details: details.map(ToString::to_string),
        }
    }
}
