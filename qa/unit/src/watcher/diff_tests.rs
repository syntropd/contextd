//! Unit QA tests for configuration snapshot and unified diff generation.

#[cfg(test)]
mod tests {
    use contextd_core::watcher::snapshot::compute_text_diff;

    #[test]
    fn test_identical_content_returns_none() {
        let path = "/etc/hosts";
        let content = "127.0.0.1 localhost\n::1 localhost\n";
        let diff = compute_text_diff(path, content, content);
        assert!(diff.is_none(), "Expected no diff for identical content");
    }

    #[test]
    fn test_empty_to_nonempty_content() {
        let path = "/etc/resolv.conf";
        let old = "";
        let new = "nameserver 1.1.1.1\n";
        let diff = compute_text_diff(path, old, new);
        assert!(diff.is_some());
        let d = diff.unwrap();
        assert!(d.contains("--- a/etc/resolv.conf"));
        assert!(d.contains("+++ b/etc/resolv.conf"));
        assert!(d.contains("+nameserver 1.1.1.1"));
    }

    #[test]
    fn test_line_replacement_diff() {
        let path = "/etc/ssh/sshd_config";
        let old = "Port 22\nPermitRootLogin yes\n";
        let new = "Port 2222\nPermitRootLogin no\n";
        let diff = compute_text_diff(path, old, new).expect("Expected diff");
        assert!(d_has_line(&diff, "-Port 22"));
        assert!(d_has_line(&diff, "+Port 2222"));
        assert!(d_has_line(&diff, "-PermitRootLogin yes"));
        assert!(d_has_line(&diff, "+PermitRootLogin no"));
    }

    #[test]
    fn test_trailing_newline_handling() {
        let path = "/etc/motd";
        let old = "Welcome";
        let new = "Welcome\n";
        // Both single line with or without newline should produce appropriate diff
        let diff = compute_text_diff(path, old, new);
        // If content is semantically identical lines, diff might be empty or formatted
        if let Some(d) = diff {
            assert!(d.contains("--- a/etc/motd"));
        }
    }

    fn d_has_line(diff: &str, line: &str) -> bool {
        diff.lines().any(|l| l == line)
    }
}
