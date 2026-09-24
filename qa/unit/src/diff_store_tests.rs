//! Unit QA tests for DiffStore on-disk repository operations.

#[cfg(test)]
mod tests {
    use contextd_core::watcher::DiffStore;
    use tempfile::tempdir;

    #[test]
    fn test_record_and_list_diff_for_path() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();

        let path = "/etc/nginx/nginx.conf";
        let diff_content = "--- a/etc/nginx/nginx.conf\n+++ b/etc/nginx/nginx.conf\n@@ -1 +1 @@\n-worker_processes 1;\n+worker_processes 4;\n";

        let record = store.record_diff(path, diff_content).unwrap();
        assert_eq!(record.file_path, path);
        assert_eq!(record.diff_content, diff_content);

        let list = store.list_diffs_for_path(path, 0).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].diff_content, diff_content);
    }

    #[test]
    fn test_list_recent_diffs_across_paths() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();

        store
            .record_diff("/etc/hosts", "--- a/etc/hosts\n+++ b/etc/hosts\n+1.1.1.1 test\n")
            .unwrap();
        store
            .record_diff("/etc/resolv.conf", "--- a/etc/resolv.conf\n+++ b/etc/resolv.conf\n+nameserver 8.8.8.8\n")
            .unwrap();

        let all = store.list_recent_diffs(0).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_time_filtering() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();

        let path = "/etc/fstab";
        let rec = store.record_diff(path, "--- a/etc/fstab\n+test").unwrap();

        // Query with since_us in the future should return empty list
        let empty = store.list_diffs_for_path(path, rec.timestamp_us + 1_000_000).unwrap();
        assert!(empty.is_empty());

        // Query with past timestamp returns record
        let found = store.list_diffs_for_path(path, rec.timestamp_us).unwrap();
        assert_eq!(found.len(), 1);
    }
}
