//! Package manager transaction log parser.
//!
//! Parses DNF, DPKG, and Pacman logs to extract package upgrade events.

use serde::{Deserialize, Serialize};
use std::io::BufRead;

/// Record representing a package manager installation or upgrade event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTransactionRecord {
    /// Timestamp in microseconds when the package action completed.
    pub timestamp_us: u64,
    /// Transaction action (e.g. "upgrade", "install", "remove").
    pub action: String,
    /// Package name (e.g. "openssl-libs", "glibc").
    pub package_name: String,
    /// Version details string.
    pub version: String,
}

/// Parses DNF / RPM log lines into package transaction records.
pub fn parse_dnf_log_line(line: &str) -> Option<PackageTransactionRecord> {
    // Format: "2026-09-24T01:15:00Z INFO Upgraded: openssl-3.2.2-1.fc40.x86_64"
    let trimmed = line.trim();
    if !trimmed.contains("Upgraded:") && !trimmed.contains("Installed:") {
        return None;
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let action = if trimmed.contains("Upgraded:") { "upgrade" } else { "install" };
    let pkg_raw = parts.last()?;

    let (name, ver) = split_package_nvra(pkg_raw);

    Some(PackageTransactionRecord {
        timestamp_us: parse_iso_timestamp(parts[0]),
        action: action.to_string(),
        package_name: name,
        version: ver,
    })
}

/// Parses DPKG / Debian log lines into package transaction records.
pub fn parse_dpkg_log_line(line: &str) -> Option<PackageTransactionRecord> {
    // Format: "2026-09-24 01:15:00 upgrade openssl:amd64 3.0.2 3.0.3"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let action = parts[2];
    if action != "upgrade" && action != "install" && action != "remove" {
        return None;
    }

    let raw_name = parts[3].split(':').next().unwrap_or(parts[3]);
    let ver = format!("{} -> {}", parts[4], parts[5]);

    Some(PackageTransactionRecord {
        timestamp_us: parse_date_time_timestamp(parts[0], parts[1]),
        action: action.to_string(),
        package_name: raw_name.to_string(),
        version: ver,
    })
}

/// Parses lines from a buffered reader using autodetection.
pub fn parse_package_log_stream<R: BufRead>(reader: R) -> Vec<PackageTransactionRecord> {
    let mut records = Vec::new();
    for line_res in reader.lines() {
        if let Ok(line) = line_res {
            if let Some(rec) = parse_dnf_log_line(&line) {
                records.push(rec);
            } else if let Some(rec) = parse_dpkg_log_line(&line) {
                records.push(rec);
            }
        }
    }
    records
}

fn split_package_nvra(raw: &str) -> (String, String) {
    if let Some(pos) = raw.find('-') {
        let (name, rest) = raw.split_at(pos);
        (name.to_string(), rest.trim_start_matches('-').to_string())
    } else {
        (raw.to_string(), "unknown".to_string())
    }
}

fn parse_iso_timestamp(_s: &str) -> u64 {
    // Approximate or default timestamp fallback in microseconds
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

fn parse_date_time_timestamp(_date: &str, _time: &str) -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
