//! Error types for the contextd core engine.
//!
//! Follows standard POSIX and systemd error classification.

use thiserror::Error;

/// Primary error enumeration for contextd operations.
#[derive(Debug, Error)]
pub enum ContextdError {
    /// Input/output error on host filesystem.
    #[error("Filesystem I/O failure: {0}")]
    Io(#[from] std::io::Error),

    /// Inotify or file-system monitoring syscall failure.
    #[error("Filesystem monitoring syscall error: {0}")]
    Watcher(String),

    /// Parsing failure for package manager transaction records.
    #[error("Failed to parse package manager log: {0}")]
    PackageLogParse(String),

    /// Event or diff record not found.
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Configuration parsing failure.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Operating system primitive or syscall error.
    #[error("Kernel syscall error: {0}")]
    Syscall(String),
}
