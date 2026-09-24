//! Unit QA tests for ContextdConfig serialization and loading.

#[cfg(test)]
mod tests {
    use contextd_core::config::{
        ContextdConfig, DEFAULT_STORAGE_PATH, DEFAULT_SOCKET_PATH,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_values() {
        let config = ContextdConfig::default();
        assert_eq!(config.storage_path.to_str().unwrap(), DEFAULT_STORAGE_PATH);
        assert_eq!(config.socket_path.to_str().unwrap(), DEFAULT_SOCKET_PATH);
        assert_eq!(config.retention_days, 30);
        assert!(!config.watched_directories.is_empty());
        assert!(!config.package_log_paths.is_empty());
    }

    #[test]
    fn test_load_from_valid_toml() {
        let tmp = tempdir().unwrap();
        let conf_file = tmp.path().join("contextd.conf");

        let toml_data = r#"
            storage_path = "/tmp/custom_contextd"
            socket_path = "/tmp/custom_contextd.sock"
            watched_directories = ["/etc/systemd", "/etc/foo"]
            package_log_paths = ["/var/log/dpkg.log"]
            retention_days = 7
        "#;
        fs::write(&conf_file, toml_data).unwrap();

        let loaded = ContextdConfig::load_or_default(&conf_file).unwrap();
        assert_eq!(loaded.storage_path.to_str().unwrap(), "/tmp/custom_contextd");
        assert_eq!(loaded.retention_days, 7);
        assert_eq!(loaded.watched_directories.len(), 2);
    }

    #[test]
    fn test_missing_file_falls_back_to_defaults() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("does_not_exist.conf");

        let loaded = ContextdConfig::load_or_default(missing).unwrap();
        assert_eq!(loaded, ContextdConfig::default());
    }
}
