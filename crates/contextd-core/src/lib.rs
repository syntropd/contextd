//! Pure Rust core engine for contextd.
//!
//! Provides filesystem configuration drift tracking, package transaction
//! log parsing, event log persistence, and causality correlation.

pub mod config;
pub mod correlator;
pub mod error;
pub mod index;
pub mod watcher;

pub use config::{
    ContextdConfig, DEFAULT_CONFIG_PATH, DEFAULT_SOCKET_PATH, DEFAULT_STORAGE_PATH,
};
pub use correlator::{
    correlate_unit_timeline, parse_dnf_log_line, parse_dpkg_log_line, parse_package_log_stream,
    PackageTransactionRecord, UnitIncidentContext,
};
pub use error::ContextdError;
pub use index::{compute_jaccard_similarity, EventStore, SystemEvent};
pub use watcher::{ConfigDiffRecord, DiffStore, FileTracker, compute_text_diff};
