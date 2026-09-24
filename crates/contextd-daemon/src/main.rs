//! Entry point for contextd: System Chronology and Causality Graph Daemon.

use anyhow::{Context, Result};
use contextd_core::config::{ContextdConfig, DEFAULT_CONFIG_PATH};
use contextd_core::correlator::{parse_package_log_stream, PackageTransactionRecord};
use contextd_core::index::EventStore;
use contextd_core::watcher::{DiffStore, FileTracker};
use contextd_daemon::activation::parse_listen_fds;
use contextd_daemon::notify::{notify_ready, notify_stopping};
use contextd_daemon::varlink::{Context1Handler, VarlinkServer};
use contextd_daemon::watcher_task::WatcherTask;
use std::fs;
use std::io::BufReader;
use std::os::unix::net::UnixListener as StdUnixListener;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting contextd (System Chronology & Causality Graph Daemon)");

    let args: Vec<String> = std::env::args().collect();
    let mut config_path =
        std::env::var("CONTEXTD_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    if let Some(pos) = args.iter().position(|a| a == "--config" || a == "-c") {
        if let Some(path) = args.get(pos + 1) {
            config_path = path.clone();
        }
    }

    let config = ContextdConfig::load_or_default(&config_path)
        .context("Failed loading configuration")?;

    fs::create_dir_all(&config.storage_path)
        .context("Failed creating storage directory")?;

    let diff_store = Arc::new(DiffStore::new(&config.storage_path)?);
    let event_store = Arc::new(EventStore::new(&config.storage_path)?);
    let file_tracker = Arc::new(FileTracker::new());

    let mut package_history = Vec::new();
    for log_path in &config.package_log_paths {
        if log_path.exists() {
            if let Ok(file) = fs::File::open(log_path) {
                let reader = BufReader::new(file);
                let records = parse_package_log_stream(reader);
                package_history.extend(records);
            }
        }
    }
    package_history.sort_by_key(|r: &PackageTransactionRecord| r.timestamp_us);
    let package_history = Arc::new(package_history);

    let handler = Context1Handler::new(
        Arc::clone(&diff_store),
        Arc::clone(&event_store),
        Arc::clone(&package_history),
    );

    let watcher = WatcherTask::new(
        Arc::clone(&file_tracker),
        Arc::clone(&diff_store),
        config.watched_directories.clone(),
        Duration::from_secs(10),
    );
    watcher.initialize_baselines();
    tokio::spawn(watcher.run());

    let activated = parse_listen_fds();
    let listener = match activated.varlink_listener {
        Some(l) => {
            info!("Adopted socket from systemd socket activation");
            l
        }
        None => {
            let socket_path = &config.socket_path;
            if let Some(parent) = socket_path.parent() {
                fs::create_dir_all(parent)?;
            }
            if socket_path.exists() {
                let _ = fs::remove_file(socket_path);
            }
            info!("Listening on Unix domain socket: {}", socket_path.display());
            let std_listener = StdUnixListener::bind(socket_path)?;
            std_listener.set_nonblocking(true)?;
            UnixListener::from_std(std_listener)?
        }
    };

    let server = VarlinkServer::new(listener, handler);

    notify_ready();
    info!("contextd successfully initialized and ready");

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        res = server.run() => {
            if let Err(e) = res {
                error!("Varlink server exited with error: {}", e);
            }
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, initiating shutdown");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT, initiating shutdown");
        }
    }

    notify_stopping();
    info!("contextd shutdown completed");
    Ok(())
}
