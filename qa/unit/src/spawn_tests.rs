//! Unit QA tests that execute the shipped binaries end to end:
//! daemon startup over a fresh socket and contextctl dispatch.

#[cfg(test)]
mod tests {
    use contextctl::client::ContextdClient;
    use serde_json::json;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    fn bin_path(name: &str) -> PathBuf {
        if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
            return PathBuf::from(dir).join("debug").join(name);
        }
        workspace_root().join("target").join("debug").join(name)
    }

    fn ensure_bins_built() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let status = Command::new("cargo")
                .current_dir(workspace_root())
                .args(["build", "--offline", "-q", "--bin", "contextd", "--bin", "contextctl"])
                .status()
                .expect("failed to run cargo build for spawn tests");
            assert!(status.success(), "cargo build of binaries failed");
        });
    }

    fn stop(mut child: Child) {
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_contextctl_completions_output_names_binary() {
        ensure_bins_built();
        let out = Command::new(bin_path("contextctl"))
            .args(["completions", "bash"])
            .output()
            .expect("failed to run contextctl");
        assert!(out.status.success());
        let script = String::from_utf8_lossy(&out.stdout);
        assert!(script.contains("contextctl"), "completion script is empty");
    }

    #[test]
    fn test_contextctl_info_without_daemon_fails() {
        ensure_bins_built();
        let out = Command::new(bin_path("contextctl"))
            .args(["--socket", "/tmp/contextd-qa-missing.sock", "info"])
            .output()
            .expect("failed to run contextctl");
        assert!(!out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("Failed to connect"), "stderr was: {stderr}");
    }

    #[tokio::test]
    async fn test_contextd_serves_requests_over_fresh_socket() {
        ensure_bins_built();
        let tmp = tempdir().unwrap();
        let storage = tmp.path().join("store");
        let socket = tmp.path().join("ctx.sock");
        let config = tmp.path().join("contextd.conf");
        std::fs::write(
            &config,
            format!(
                "storage_path = '{}'\nsocket_path = '{}'\nwatched_directories = []\npackage_log_paths = []\nretention_days = 1\n",
                storage.display(),
                socket.display()
            ),
        )
        .unwrap();

        let child = Command::new(bin_path("contextd"))
            .args(["--config", &config.to_string_lossy()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn contextd");

        let client = ContextdClient::new(&socket);
        let deadline = Instant::now() + Duration::from_secs(15);
        let info = loop {
            match client.call("org.varlink.service.GetInfo", None).await {
                Ok(info) => break info,
                Err(_) if Instant::now() < deadline => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    stop(child);
                    panic!("daemon never answered GetInfo: {e}");
                }
            }
        };
        assert_eq!(info["product"], "contextd");

        let rec = client
            .call(
                "io.syntrop.Context1.RecordEvent",
                Some(json!({"source": "spawn-qa", "summary": "hello"})),
            )
            .await;
        match rec {
            Ok(v) => assert!(v.get("event_id").is_some()),
            Err(e) => {
                stop(child);
                panic!("RecordEvent failed: {e}");
            }
        }
        stop(child);
    }
}