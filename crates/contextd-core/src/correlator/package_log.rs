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
        timestamp_us: parse_iso8601_micros(parts[0]),
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
        timestamp_us: parse_dpkg_micros(parts[0], parts[1]),
        action: action.to_string(),
        package_name: raw_name.to_string(),
        version: ver,
    })
}

/// Parses a Pacman log line into a package transaction record.
///
/// Format: `[2024-09-25T03:54:25+0000] [PACMAN] upgraded openssl (3.0.2-1 -> 3.0.3-1)`
/// Variants cover `upgraded`, `installed`, `removed`.
pub fn parse_pacman_log_line(line: &str) -> Option<PackageTransactionRecord> {
    let trimmed = line.trim();
    let bytes = trimmed.as_bytes();
    if bytes.first() != Some(&b'[') || !trimmed.contains("] [PACMAN] ") {
        return None;
    }
    let close_bracket = trimmed.find(']')?;
    let ts_raw = &trimmed[1..close_bracket];
    let tail = trimmed[close_bracket + 1..].trim_start();
    let action_keyword = tail.strip_prefix("[PACMAN] ")?.trim_start();
    let (action, rest) = if let Some(s) = action_keyword.strip_prefix("upgraded ") {
        ("upgrade", s)
    } else if let Some(s) = action_keyword.strip_prefix("installed ") {
        ("install", s)
    } else if let Some(s) = action_keyword.strip_prefix("removed ") {
        ("remove", s)
    } else {
        return None;
    };

    let open = rest.find('(')?;
    let close = rest.rfind(')')?;
    if close <= open {
        return None;
    }
    let name = rest[..open].trim().to_string();
    let version = rest[open + 1..close].to_string();
    if name.is_empty() || version.is_empty() {
        return None;
    }

    Some(PackageTransactionRecord {
        timestamp_us: parse_iso8601_micros(ts_raw),
        action: action.to_string(),
        package_name: name,
        version,
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
            } else if let Some(rec) = parse_pacman_log_line(&line) {
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

/// Returns the microseconds-since-epoch for an ISO 8601 UTC timestamp
/// string (with or without the trailing `Z`), or `0` if the input is
/// malformed. Malformed timestamps must not fail the parse — they
/// just get a sentinel of `0`.
fn parse_iso8601_micros(s: &str) -> u64 {
    super::timestamp::parse_iso8601_utc(s).unwrap_or(0)
}

/// Returns the microseconds-since-epoch for a `YYYY-MM-DD` + `HH:MM:SS`
/// pair (as emitted by DPKG logs), or `0` if the input is malformed.
fn parse_dpkg_micros(date: &str, time: &str) -> u64 {
    super::timestamp::parse_dpkg_date_time(date, time).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pacman_round_trip() {
        let line = "[2024-09-25T03:54:25+0000] [PACMAN] upgraded openssl (3.0.2-1 -> 3.0.3-1)";
        let rec = parse_pacman_log_line(line).unwrap();
        assert_eq!(rec.action, "upgrade");
        assert_eq!(rec.package_name, "openssl");
        assert_eq!(rec.version, "3.0.2-1 -> 3.0.3-1");
        // 2024-09-25T03:54:25Z == 1727236465s
        assert_eq!(rec.timestamp_us, 1_727_236_465 * 1_000_000);
    }

    #[test]
    fn test_parse_pacman_installed() {
        let line = "[2024-09-25T03:54:25+0000] [PACMAN] installed vim (9.0.1378-2)";
        let rec = parse_pacman_log_line(line).unwrap();
        assert_eq!(rec.action, "install");
        assert_eq!(rec.package_name, "vim");
        assert_eq!(rec.version, "9.0.1378-2");
    }

    #[test]
    fn test_parse_pacman_removed() {
        let line = "[2024-09-25T03:54:25+0000] [PACMAN] removed nano (7.2-1)";
        let rec = parse_pacman_log_line(line).unwrap();
        assert_eq!(rec.action, "remove");
        assert_eq!(rec.package_name, "nano");
    }

    #[test]
    fn test_parse_pacman_garbage_returns_none() {
        assert!(parse_pacman_log_line("not a pacman line").is_none());
        assert!(parse_pacman_log_line("").is_none());
    }
}
