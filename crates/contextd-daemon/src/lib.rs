//! Core daemon service implementation for contextd.

pub mod activation;
pub mod notify;
pub mod varlink;
pub mod watcher_task;

pub use activation::{parse_listen_fds, ActivatedSockets};
pub use notify::{notify_ready, notify_status, notify_stopping, notify_watchdog, send_notify};
pub use varlink::{Context1Handler, VarlinkCall, VarlinkReply, VarlinkServer};
pub use watcher_task::WatcherTask;
