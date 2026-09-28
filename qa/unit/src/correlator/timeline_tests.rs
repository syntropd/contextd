//! Unit QA tests for causality timeline correlation.

#[cfg(test)]
mod tests {
    use contextd_core::correlator::{correlate_unit_timeline, PackageTransactionRecord};
    use contextd_core::watcher::DiffStore;
    use tempfile::tempdir;

    #[test]
    fn test_correlate_unit_with_matching_diff() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();

        // Record a diff that contains "nginx"
        store
            .record_diff(
                "/etc/nginx/conf.d/default.conf",
                "--- a/etc/nginx/conf.d/default.conf\n+listen 8080;",
            )
            .unwrap();

        // Record unrelated diff
        store
            .record_diff("/etc/ssh/sshd_config", "--- a/etc/ssh/sshd_config\n+Port 22")
            .unwrap();

        let timeline = correlate_unit_timeline("nginx.service", &store, &[], 0);

        assert_eq!(timeline.unit_name, "nginx.service");
        assert_eq!(timeline.config_diffs.len(), 1);
        assert!(timeline.summary.contains("configuration modification"));
    }

    #[test]
    fn test_correlate_unit_with_package_upgrade() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();

        let pkgs = vec![PackageTransactionRecord {
            timestamp_us: 1_000_000,
            action: "upgrade".to_string(),
            package_name: "openssl".to_string(),
            version: "3.0.13".to_string(),
        }];

        let timeline = correlate_unit_timeline("sshd.service", &store, &pkgs, 500_000);

        assert_eq!(timeline.package_upgrades.len(), 1);
        assert!(timeline.summary.contains("package upgrade"));
    }

    #[test]
    fn test_correlate_unit_clean_baseline() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();

        let timeline = correlate_unit_timeline("systemd-journald.service", &store, &[], 0);

        assert!(timeline.config_diffs.is_empty());
        assert!(timeline.package_upgrades.is_empty());
        assert!(timeline.summary.contains("No recent configuration drift"));
    }
}
