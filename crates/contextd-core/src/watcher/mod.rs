//! Filesystem configuration drift and change detection subsystem.
//!
//! Provides unified text diff calculation, persistent diff storage,
//! and revision tracking.

pub mod diff_store;
pub mod file_tracker;
pub mod snapshot;

pub use diff_store::{ConfigDiffRecord, DiffStore};
pub use file_tracker::FileTracker;
pub use snapshot::compute_text_diff;
