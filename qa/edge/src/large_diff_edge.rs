//! Edge tests for large diff generation and high-volume changes.

#[cfg(test)]
mod tests {
    use contextd_core::watcher::snapshot::compute_text_diff;
    use contextd_core::watcher::DiffStore;
    use tempfile::tempdir;

    #[test]
    fn test_large_file_diff_handling() {
        // Generate 5,000 lines of configuration
        let mut old_lines = Vec::with_capacity(5000);
        let mut new_lines = Vec::with_capacity(5000);

        for i in 0..5000 {
            old_lines.push(format!("rule_{}_enabled=true", i));
            if i % 100 == 0 {
                new_lines.push(format!("rule_{}_enabled=false", i));
            } else {
                new_lines.push(format!("rule_{}_enabled=true", i));
            }
        }

        let old_content = old_lines.join("\n");
        let new_content = new_lines.join("\n");

        let diff = compute_text_diff("/etc/huge.conf", &old_content, &new_content);
        assert!(diff.is_some());
        let diff_str = diff.unwrap();
        assert!(diff_str.contains("rule_0_enabled=false"));
        assert!(diff_str.contains("rule_100_enabled=false"));
    }

    #[test]
    fn test_diff_store_with_many_revisions() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();
        let path = "/etc/rapid_change.conf";

        for i in 0..100 {
            let diff = format!("--- a/etc/rapid_change.conf\n+version={}", i);
            store.record_diff(path, &diff).unwrap();
        }

        let diffs = store.list_diffs_for_path(path, 0).unwrap();
        assert_eq!(diffs.len(), 100);
    }
}
