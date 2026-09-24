//! Causality correlation subsystem.
//!
//! Correlates package manager transactions, configuration file diffs,
//! and timeline events for unit crash triage.

pub mod package_log;
pub mod timeline;

pub use package_log::{
    parse_dnf_log_line, parse_dpkg_log_line, parse_package_log_stream, PackageTransactionRecord,
};
pub use timeline::{correlate_unit_timeline, UnitIncidentContext};
