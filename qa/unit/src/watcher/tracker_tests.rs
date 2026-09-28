//! Unit QA tests for FileTracker state tracking.

#[cfg(test)]
mod tests {
    use contextd_core::watcher::{DiffStore, FileTracker};
    use tempfile::tempdir;

    #[test]
    fn test_tracker_detects_content_drift() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();
        let tracker = FileTracker::new();

        let path = "/etc/test.conf";
        tracker.set_baseline(path, "key=val1\n");

        // No change
        let no_diff = tracker.check_and_record(path, "key=val1\n", &store).unwrap();
        assert!(no_diff.is_none());

        // Modification
        let diff = tracker.check_and_record(path, "key=val2\n", &store).unwrap();
        assert!(diff.is_some());
        let record = diff.unwrap();
        assert_eq!(record.file_path, path);
        assert!(record.diff_content.contains("-key=val1"));
        assert!(record.diff_content.contains("+key=val2"));

        // Subsequent check with same content returns no diff
        let subsequent = tracker.check_and_record(path, "key=val2\n", &store).unwrap();
        assert!(subsequent.is_none());
    }

    #[test]
    fn test_tracker_inspect_disk_file() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();
        let tracker = FileTracker::new();

        let file_path = tmp.path().join("sample.conf");
        std::fs::write(&file_path, "initial=1\n").unwrap();

        tracker.set_baseline(&file_path.to_string_lossy(), "initial=1\n");

        // Update file on disk
        std::fs::write(&file_path, "initial=2\n").unwrap();

        let diff = tracker.inspect_disk_file(&file_path, &store).unwrap();
        assert!(diff.is_some());
        assert!(diff.unwrap().diff_content.contains("+initial=2"));
    }
}
