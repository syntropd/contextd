//! Configuration subsystem for contextd.

pub mod contextd_config;

pub use contextd_config::{
    ContextdConfig, DEFAULT_CONFIG_PATH, DEFAULT_SOCKET_PATH, DEFAULT_STORAGE_PATH,
};
