//! Edge tests for the WatcherTask recursive directory walker and
//! large-file handling.

#[cfg(test)]
mod tests {
    use contextd_core::watcher::{DiffStore, FileTracker};
    use contextd_daemon::watcher_task::walk_dir;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_recursive_scan_finds_files_in_subdirs() {
        let tmp = tempdir().unwrap();
        let sub = tmp.path().join("nested/deeper");
        fs::create_dir_all(&sub).unwrap();
        let f1 = tmp.path().join("top.conf");
        let f2 = sub.join("leaf.yaml");
        fs::write(&f1, "a=1\n").unwrap();
        fs::write(&f2, "b=2\n").unwrap();

        let mut visited: Vec<PathBuf> = Vec::new();
        walk_dir(tmp.path(), &mut |p| visited.push(p.to_path_buf()));
        visited.sort();
        assert!(
            visited.iter().any(|p| p == &f1),
            "top-level file missing from walk: {:?}",
            visited
        );
        assert!(
            visited.iter().any(|p| p == &f2),
            "nested file missing from walk: {:?}",
            visited
        );
    }

    #[test]
    fn test_walk_dir_skips_symlinks_and_breaks_cycles() {
        let tmp = tempdir().unwrap();
        // Create a self-referential symlink inside the tree.
        let sub = tmp.path().join("loop");
        fs::create_dir_all(&sub).unwrap();
        std::os::unix::fs::symlink(tmp.path(), sub.join("parent")).unwrap();

        let mut visited: Vec<PathBuf> = Vec::new();
        walk_dir(tmp.path(), &mut |p| visited.push(p.to_path_buf()));
        // The walker must terminate (no infinite recursion) and must not
        // report any path through the symlink.
        assert!(
            visited.iter().all(|p| !p.ends_with("parent")),
            "walker followed symlink into cycle: {:?}",
            visited
        );
    }

    #[test]
    fn test_oversized_file_is_skipped() {
        let tmp = tempdir().unwrap();
        let store = DiffStore::new(tmp.path()).unwrap();
        let tracker = FileTracker::with_max_file_bytes(8);
        let big = "x".repeat(64);
        let res = tracker.check_and_record("/etc/big.conf", &big, &store);
        assert!(res.is_err(), "oversized file should be rejected");
    }
}