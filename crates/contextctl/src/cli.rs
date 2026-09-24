//! Command-line argument parser definitions for contextctl.

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use contextd_core::config::DEFAULT_SOCKET_PATH;
use std::path::PathBuf;

/// CLI client for contextd system chronology and causality daemon.
#[derive(Parser, Debug)]
#[command(
    name = "contextctl",
    version,
    about = "Control and query the contextd causality daemon",
    long_about = "Query configuration drift, package upgrades, and unit incident chronology via Varlink."
)]
pub struct Cli {
    /// Path to the contextd Varlink Unix domain socket.
    /// Uses `--socket`/`-S` (uppercase) to avoid colliding with
    /// `record --source`/`-s` on the global parser.
    #[arg(short = 'S', long = "socket", default_value = DEFAULT_SOCKET_PATH, global = true)]
    pub socket: PathBuf,

    /// Output results in formatted JSON.
    #[arg(long = "json", global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for contextctl.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Retrieve causality timeline and configuration drift for a unit.
    Unit {
        /// Systemd unit name to inspect (e.g. nginx.service).
        unit: String,

        /// Time window in seconds to inspect historical drift.
        #[arg(short = 'w', long = "since", default_value = "3600")]
        since: u64,
    },

    /// List recent system configuration diffs.
    Diffs {
        /// Time window in seconds to search for diffs.
        #[arg(short = 'w', long = "since", default_value = "3600")]
        since: u64,
    },

    /// Query historical system events recorded by contextd.
    Events {
        /// Filter events by systemd unit name.
        #[arg(short = 'u', long = "unit")]
        unit: Option<String>,

        /// Time window in seconds.
        #[arg(short = 'w', long = "since", default_value = "86400")]
        since: u64,

        /// Maximum number of events to return.
        #[arg(short = 'l', long = "limit", default_value = "100")]
        limit: usize,
    },

    /// Record a system event into the causality event store.
    Record {
        /// Subsystem or agent recording the event.
        #[arg(short = 's', long = "source")]
        source: String,

        /// Associated systemd unit name, if applicable.
        #[arg(short = 'u', long = "unit")]
        unit: Option<String>,

        /// High-level event summary.
        #[arg(short = 'm', long = "summary")]
        summary: String,

        /// Optional detailed context or diagnostic payload.
        #[arg(short = 'd', long = "details")]
        details: Option<String>,
    },

    /// Query daemon vendor info and interface descriptions.
    Info,

    /// Generate shell auto-completion script.
    Completions {
        /// Target shell for auto-completion.
        #[arg(value_enum)]
        shell: Shell,
    },
}
