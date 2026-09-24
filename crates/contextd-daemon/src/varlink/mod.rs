//! Varlink protocol definitions and server implementation.

pub mod context1;
pub mod protocol;
pub mod server;
pub mod service;

pub use context1::Context1Handler;
pub use protocol::{VarlinkCall, VarlinkReply};
pub use server::VarlinkServer;
pub use service::{handle_service_call, IO_SYNTROP_CONTEXT1_INTERFACE};
