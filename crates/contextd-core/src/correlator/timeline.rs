//! Chronological causality timeline correlation.
//!
//! Aggregates recent configuration drift, package upgrades, and events for a unit.

use crate::correlator::package_log::PackageTransactionRecord;
use crate::watcher::diff_store::{ConfigDiffRecord, DiffStore};
use serde::{Deserialize, Serialize};

/// Comprehensive incident correlation context for a target systemd unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitIncidentContext {
    /// Name of the target systemd unit (e.g. "nginx.service").
    pub unit_name: String,
    /// Relevant configuration file diffs that occurred before the incident.
    pub config_diffs: Vec<ConfigDiffRecord>,
    /// Recent package manager upgrades.
    pub package_upgrades: Vec<PackageTransactionRecord>,
    /// Synthesized causality summary for sentry triage.
    pub summary: String,
}

/// Builds an incident correlation context by inspecting recent drift.
pub fn correlate_unit_timeline(
    unit_name: &str,
    diff_store: &DiffStore,
    package_history: &[PackageTransactionRecord],
    since_us: u64,
) -> UnitIncidentContext {
    let all_diffs = diff_store.list_recent_diffs(since_us).unwrap_or_default();

    // Filter diffs relevant to this unit (e.g. "nginx" matches "/etc/nginx/...")
    let unit_base = unit_name.strip_suffix(".service").unwrap_or(unit_name);
    let relevant_diffs: Vec<ConfigDiffRecord> = all_diffs
        .into_iter()
        .filter(|d| d.file_path.contains(unit_base))
        .collect();

    let relevant_pkgs: Vec<PackageTransactionRecord> = package_history
        .iter()
        .filter(|p| p.timestamp_us >= since_us)
        .cloned()
        .collect();

    let summary = if !relevant_diffs.is_empty() {
        format!(
            "Detected {} configuration modification(s) affecting {} within inspection window.",
            relevant_diffs.len(),
            unit_name
        )
    } else if !relevant_pkgs.is_empty() {
        format!(
            "Detected {} package upgrade(s) completed recently before unit failure.",
            relevant_pkgs.len()
        )
    } else {
        format!("No recent configuration drift or package upgrades detected for {}.", unit_name)
    };

    UnitIncidentContext {
        unit_name: unit_name.to_string(),
        config_diffs: relevant_diffs,
        package_upgrades: relevant_pkgs,
        summary,
    }
}
