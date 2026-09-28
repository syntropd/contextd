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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_names_each_variant() {
        assert_eq!(
            ContextdError::Watcher("inotify".to_string()).to_string(),
            "Filesystem monitoring syscall error: inotify"
        );
        assert_eq!(
            ContextdError::PackageLogParse("bad line".to_string()).to_string(),
            "Failed to parse package manager log: bad line"
        );
        assert_eq!(
            ContextdError::NotFound("evt-1".to_string()).to_string(),
            "Record not found: evt-1"
        );
        assert_eq!(
            ContextdError::Config("missing key".to_string()).to_string(),
            "Configuration error: missing key"
        );
        assert_eq!(
            ContextdError::Syscall("EPERM".to_string()).to_string(),
            "Kernel syscall error: EPERM"
        );
    }

    #[test]
    fn test_io_converts_with_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err = ContextdError::from(io_err);
        assert!(matches!(err, ContextdError::Io(_)));
        assert!(err.to_string().starts_with("Filesystem I/O failure: "));
    }
}
