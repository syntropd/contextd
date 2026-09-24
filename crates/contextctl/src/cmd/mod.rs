//! Subcommand implementations for contextctl.

pub mod completions_cmd;
pub mod diff_cmd;
pub mod events_cmd;
pub mod info_cmd;
pub mod record_cmd;
pub mod unit_cmd;

pub use completions_cmd::exec_completions;
pub use diff_cmd::exec_diffs;
pub use events_cmd::exec_events;
pub use info_cmd::exec_info;
pub use record_cmd::exec_record;
pub use unit_cmd::exec_unit;
