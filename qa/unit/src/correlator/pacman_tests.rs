//! Unit QA tests for Pacman package manager log parsing.

#[cfg(test)]
mod tests {
    use contextd_core::correlator::parse_pacman_log_line;

    #[test]
    fn test_parse_pacman_upgraded() {
        let line = "[2024-09-25T03:54:25+0000] [PACMAN] upgraded openssl (3.0.2-1 -> 3.0.3-1)";
        let rec = parse_pacman_log_line(line).unwrap();
        assert_eq!(rec.action, "upgrade");
        assert_eq!(rec.package_name, "openssl");
        assert_eq!(rec.version, "3.0.2-1 -> 3.0.3-1");
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