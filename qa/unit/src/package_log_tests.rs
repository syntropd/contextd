//! Unit QA tests for package manager log parsing.

#[cfg(test)]
mod tests {
    use contextd_core::correlator::{
        parse_dnf_log_line, parse_dpkg_log_line, parse_package_log_stream,
    };
    use std::io::Cursor;

    #[test]
    fn test_parse_dnf_installed_line() {
        let line = "2024-09-24T01:23:45Z SUBDEBUG Installed: nginx-1.24.0-1.fc40.x86_64";
        let record = parse_dnf_log_line(line);
        assert!(record.is_some());
        let r = record.unwrap();
        assert_eq!(r.package_name, "nginx");
        assert_eq!(r.action, "install");
        assert_eq!(r.version, "1.24.0-1.fc40.x86_64");
    }

    #[test]
    fn test_parse_dnf_upgraded_line() {
        let line = "2024-09-24T02:00:00Z SUBDEBUG Upgraded: systemd-255.4-1.fc40.x86_64";
        let record = parse_dnf_log_line(line);
        assert!(record.is_some());
        let r = record.unwrap();
        assert_eq!(r.package_name, "systemd");
        assert_eq!(r.action, "upgrade");
    }

    #[test]
    fn test_parse_dpkg_upgrade_line() {
        let line = "2024-09-24 10:15:30 upgrade curl:amd64 7.88.1-10+deb12u5 7.88.1-10+deb12u6";
        let record = parse_dpkg_log_line(line);
        assert!(record.is_some());
        let r = record.unwrap();
        assert_eq!(r.package_name, "curl");
        assert_eq!(r.action, "upgrade");
        assert_eq!(r.version, "7.88.1-10+deb12u5 -> 7.88.1-10+deb12u6");
    }

    #[test]
    fn test_parse_stream_mixed_lines() {
        let data = "random unparseable line\n\
                    2024-09-24T01:00:00Z SUBDEBUG Installed: bash-5.2.26-1.fc40.x86_64\n\
                    another comment\n\
                    2024-09-24 08:00:00 install vim:amd64 <none> 2:9.0.1378-2\n";

        let cursor = Cursor::new(data);
        let records = parse_package_log_stream(cursor);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].package_name, "bash");
        assert_eq!(records[1].package_name, "vim");
    }
}
