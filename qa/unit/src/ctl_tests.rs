//! Unit QA tests for the contextctl CLI surface: argument parsing,
//! Varlink client calls, and every command handler.

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};
    use contextctl::cli::{exec_completions, Cli, Commands};
    use contextctl::client::ContextdClient;
    use contextctl::cmd::*;
    use serde_json::{json, Value};
    use std::path::PathBuf;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{UnixListener, UnixStream};

    /// Canned Varlink replies keyed by method name.
    fn canned_reply(method: &str, params: Option<&Value>) -> Value {
        let ok = |parameters: Value| json!({ "parameters": parameters });
        match method {
            "io.syntrop.Context1.GetUnitContext" => ok(json!({
                "context": {
                    "summary": "unit context for tests",
                    "config_diffs": [
                        {"file_path": "/etc/test.conf", "timestamp_us": 1, "diff_content": "-a\n+b\n"}
                    ],
                    "package_upgrades": [
                        {"package_name": "openssl", "action": "upgrade", "version": "3.0.3-1"}
                    ]
                }
            })),
            "io.syntrop.Context1.ListRecentDiffs" => {
                let empty = params
                    .and_then(|p| p.get("since_seconds"))
                    .and_then(|s| s.as_u64())
                    == Some(0);
                if empty {
                    ok(json!({ "diffs": [] }))
                } else {
                    ok(json!({ "diffs": [
                        {"file_path": "/etc/a.conf", "timestamp_us": 7, "diff_content": "-x\n+y\n"}
                    ] }))
                }
            }
            "io.syntrop.Context1.ListEvents" => {
                let empty = params
                    .and_then(|p| p.get("limit"))
                    .and_then(|l| l.as_u64())
                    == Some(0);
                if empty {
                    ok(json!({ "events": [] }))
                } else {
                    ok(json!({ "events": [
                        {"id": "evt-1", "source": "sentry", "unit": "a.service", "summary": "boom"}
                    ] }))
                }
            }
            "io.syntrop.Context1.RecordEvent" => ok(json!({ "event_id": "evt-42" })),
            "org.varlink.service.GetInfo" => ok(json!({
                "vendor": "Test", "product": "contextd", "version": "0.0.0",
                "url": "https://example.invalid",
                "interfaces": ["org.varlink.service", "io.syntrop.Context1"]
            })),
            "org.varlink.service.GetInterfaceDescription" => {
                ok(json!({ "description": "interface io.syntrop.Context1\nmethod Ping() -> ()" }))
            }
            "test.ErrorBare" => json!({ "error": "test.BareError" }),
            "test.ErrorDetailed" => {
                json!({ "error": "test.Detailed", "parameters": { "reason": "nope" } })
            }
            _ => json!({
                "error": "org.varlink.service.MethodNotFound",
                "parameters": { "method": method }
            }),
        }
    }

    async fn serve_one(mut stream: UnixStream) {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            match stream.read(&mut chunk).await {
                Ok(0) => return,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buf.iter().position(|&b| b == 0x00) {
                        let req: Value =
                            serde_json::from_slice(&buf[..pos]).unwrap_or(Value::Null);
                        let method =
                            req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        let reply = canned_reply(method, req.get("parameters"));
                        let mut bytes = serde_json::to_vec(&reply).unwrap();
                        bytes.push(0x00);
                        let _ = stream.write_all(&bytes).await;
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    }

    /// Serves canned replies on a temp socket until the handle is aborted.
    async fn spawn_fake_daemon() -> (tempfile::TempDir, PathBuf, tokio::task::JoinHandle<()>) {
        let tmp = tempdir().unwrap();
        let sock = tmp.path().join("fake.sock");
        let listener = UnixListener::bind(&sock).unwrap();
        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        tokio::spawn(serve_one(stream));
                    }
                    Err(_) => return,
                }
            }
        });
        (tmp, sock, handle)
    }

    #[test]
    fn test_cli_parses_every_subcommand() {
        let cli = Cli::try_parse_from(["contextctl", "unit", "nginx.service", "-w", "60"]).unwrap();
        assert!(matches!(cli.command, Commands::Unit { .. }));
        let cli = Cli::try_parse_from(["contextctl", "diffs"]).unwrap();
        assert!(matches!(cli.command, Commands::Diffs { .. }));
        let cli =
            Cli::try_parse_from(["contextctl", "events", "-u", "a.service", "-l", "5"]).unwrap();
        assert!(matches!(cli.command, Commands::Events { .. }));
        let cli = Cli::try_parse_from([
            "contextctl", "record", "-s", "sentry", "-m", "boom", "-d", "detail",
        ])
        .unwrap();
        assert!(matches!(cli.command, Commands::Record { .. }));
        assert!(Cli::try_parse_from(["contextctl", "record", "-s", "x"]).is_err());
        let cli = Cli::try_parse_from(["contextctl", "info"]).unwrap();
        assert!(matches!(cli.command, Commands::Info));
        let cli = Cli::try_parse_from(["contextctl", "completions", "bash"]).unwrap();
        assert!(matches!(cli.command, Commands::Completions { .. }));
    }

    #[test]
    fn test_cli_parses_global_flags() {
        let cli =
            Cli::try_parse_from(["contextctl", "-S", "/tmp/x.sock", "--json", "info"]).unwrap();
        assert_eq!(cli.socket, PathBuf::from("/tmp/x.sock"));
        assert!(cli.json);
    }

    #[test]
    fn test_exec_completions_runs() {
        let cli = Cli::try_parse_from(["contextctl", "completions", "bash"]).unwrap();
        let Commands::Completions { shell } = cli.command else {
            panic!("expected Completions");
        };
        exec_completions(&mut Cli::command(), shell);
    }

    #[tokio::test]
    async fn test_client_call_returns_parameters() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        let res = client.call("org.varlink.service.GetInfo", None).await.unwrap();
        assert_eq!(res["product"], "contextd");
        server.abort();
    }

    #[tokio::test]
    async fn test_client_call_surfaces_errors() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        let err = client.call("test.ErrorBare", None).await.unwrap_err();
        assert_eq!(err.to_string(), "Varlink error returned: test.BareError");
        let err = client.call("test.ErrorDetailed", None).await.unwrap_err();
        assert!(err.to_string().contains("test.Detailed"));
        assert!(err.to_string().contains("nope"));
        let err = client.call("no.Such.Method", None).await.unwrap_err();
        assert!(err.to_string().contains("MethodNotFound"));
        server.abort();
    }

    #[tokio::test]
    async fn test_client_call_to_missing_socket_fails() {
        let client = ContextdClient::new("/tmp/contextd-qa-missing.sock");
        let err = client.call("org.varlink.service.GetInfo", None).await.unwrap_err();
        assert!(err.to_string().contains("Failed to connect"));
    }

    #[tokio::test]
    async fn test_exec_unit_json_and_text() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        exec_unit(&client, "a.service", 60, true).await.unwrap();
        exec_unit(&client, "a.service", 60, false).await.unwrap();
        server.abort();
    }

    #[tokio::test]
    async fn test_exec_diffs_json_text_and_empty() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        exec_diffs(&client, 60, true).await.unwrap();
        exec_diffs(&client, 60, false).await.unwrap();
        exec_diffs(&client, 0, false).await.unwrap();
        server.abort();
    }

    #[tokio::test]
    async fn test_exec_events_json_text_and_empty() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        exec_events(&client, Some("a.service"), 60, 10, true).await.unwrap();
        exec_events(&client, None, 60, 10, false).await.unwrap();
        exec_events(&client, None, 60, 0, false).await.unwrap();
        server.abort();
    }

    #[tokio::test]
    async fn test_exec_record_with_and_without_optionals() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        exec_record(&client, "sentry", Some("a.service"), "boom", Some("detail"))
            .await
            .unwrap();
        exec_record(&client, "sentry", None, "boom", None).await.unwrap();
        server.abort();
    }

    #[tokio::test]
    async fn test_exec_info_json_and_text() {
        let (_tmp, sock, server) = spawn_fake_daemon().await;
        let client = ContextdClient::new(&sock);
        exec_info(&client, true).await.unwrap();
        exec_info(&client, false).await.unwrap();
        server.abort();
    }
}