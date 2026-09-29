//! Unit QA tests for daemon activation, the Varlink accept loop,
//! and the watcher scan cycle.

#[cfg(test)]
mod tests {
    use contextctl::client::ContextdClient;
    use contextd_core::index::EventStore;
    use contextd_core::watcher::{DiffStore, FileTracker};
    use contextd_daemon::activation::parse_listen_fds;
    use contextd_daemon::varlink::{Context1Handler, VarlinkServer};
    use contextd_daemon::watcher_task::WatcherTask;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{UnixListener, UnixStream};

    #[test]
    fn test_parse_listen_fds_ignores_missing_or_empty_activation() {
        let saved_pid = std::env::var("LISTEN_PID").ok();
        let saved_fds = std::env::var("LISTEN_FDS").ok();

        std::env::remove_var("LISTEN_PID");
        std::env::remove_var("LISTEN_FDS");
        assert!(parse_listen_fds().varlink_listener.is_none());

        std::env::set_var("LISTEN_PID", std::process::id().to_string());
        std::env::set_var("LISTEN_FDS", "0");
        assert!(parse_listen_fds().varlink_listener.is_none());

        std::env::set_var("LISTEN_PID", "1");
        std::env::set_var("LISTEN_FDS", "1");
        assert!(parse_listen_fds().varlink_listener.is_none());

        match saved_pid {
            Some(v) => std::env::set_var("LISTEN_PID", v),
            None => std::env::remove_var("LISTEN_PID"),
        }
        match saved_fds {
            Some(v) => std::env::set_var("LISTEN_FDS", v),
            None => std::env::remove_var("LISTEN_FDS"),
        }
    }

    async fn spawn_real_server(sock: &std::path::Path) -> tokio::task::JoinHandle<()> {
        let tmp = tempdir().unwrap();
        let diff_store = Arc::new(DiffStore::new(tmp.path()).unwrap());
        let event_store = Arc::new(EventStore::new(tmp.path()).unwrap());
        let handler = Context1Handler::new(diff_store, event_store, Arc::new(vec![]));
        let listener = UnixListener::bind(sock).unwrap();
        let server = VarlinkServer::new(listener, handler);
        tokio::spawn(async move {
            let _tmp = tmp;
            let _ = server.run().await;
        })
    }

    #[tokio::test]
    async fn test_varlink_server_serves_context1_round_trip() {
        let tmp = tempdir().unwrap();
        let sock = tmp.path().join("ctx.sock");
        let server = spawn_real_server(&sock).await;
        let client = ContextdClient::new(&sock);

        let info = client
            .call("org.varlink.service.GetInfo", None)
            .await
            .unwrap();
        assert_eq!(info["product"], "contextd");

        let rec = client
            .call(
                "io.syntrop.Context1.RecordEvent",
                Some(json!({"source": "qa", "summary": "hello"})),
            )
            .await
            .unwrap();
        assert!(rec.get("event_id").is_some());

        let events = client
            .call(
                "io.syntrop.Context1.ListEvents",
                Some(json!({"since_seconds": 3600, "limit": 10})),
            )
            .await
            .unwrap();
        assert_eq!(events["events"].as_array().unwrap().len(), 1);

        let diffs = client
            .call(
                "io.syntrop.Context1.ListRecentDiffs",
                Some(json!({"since_seconds": 3600})),
            )
            .await
            .unwrap();
        assert_eq!(diffs["diffs"].as_array().unwrap().len(), 0);

        let ctx = client
            .call(
                "io.syntrop.Context1.GetUnitContext",
                Some(json!({"unit": "a.service", "since_seconds": 60})),
            )
            .await
            .unwrap();
        assert!(ctx.get("context").is_some());

        let err = client.call("no.Such.Method", None).await.unwrap_err();
        assert!(err.to_string().contains("MethodNotFound"));
        server.abort();
    }

    #[tokio::test]
    async fn test_varlink_server_rejects_malformed_call() {
        let tmp = tempdir().unwrap();
        let sock = tmp.path().join("ctx.sock");
        let server = spawn_real_server(&sock).await;

        let mut stream = UnixStream::connect(&sock).await.unwrap();
        stream.write_all(b"{not json\x00").await.unwrap();
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let n = stream.read(&mut chunk).await.unwrap();
            assert!(n > 0, "server closed connection without a reply");
            buf.extend_from_slice(&chunk[..n]);
            if buf.contains(&0x00) {
                break;
            }
        }
        let reply: serde_json::Value = serde_json::from_slice(&buf[..buf.len() - 1]).unwrap();
        assert_eq!(reply["error"], "org.varlink.service.InvalidParameter");
        server.abort();
    }

    #[test]
    fn test_watcher_scan_cycle_records_drift() {
        let tmp = tempdir().unwrap();
        let store = Arc::new(DiffStore::new(tmp.path()).unwrap());
        let tracker = Arc::new(FileTracker::new());
        let watched = tmp.path().join("watched.conf");
        std::fs::write(&watched, "a=1\n").unwrap();

        let task = WatcherTask::new(
            Arc::clone(&tracker),
            Arc::clone(&store),
            vec![watched.clone()],
            Duration::from_secs(600),
        );
        task.initialize_baselines();
        assert!(store.list_recent_diffs(0).unwrap().is_empty());

        std::fs::write(&watched, "a=2\n").unwrap();
        task.scan_cycle();
        let diffs = store.list_recent_diffs(0).unwrap();
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].file_path.ends_with("watched.conf"));
    }
}
