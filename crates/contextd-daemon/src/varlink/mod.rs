//! Varlink protocol definitions and server implementation.
//!
//! Provides the primary IPC transport for contextual memory and state,
//! secured via kernel-verified SO_PEERCRED authorization.

pub mod auth;
pub mod context1;
pub mod protocol;
pub mod server;
pub mod service;

pub use auth::{authorize_peer, lookup_group, TrustedGroup, UNRESOLVED_GID};
pub use context1::Context1Handler;
pub use protocol::{VarlinkCall, VarlinkReply};
pub use server::VarlinkServer;
pub use service::{handle_service_call, IO_SYNTROP_CONTEXT1_INTERFACE};
