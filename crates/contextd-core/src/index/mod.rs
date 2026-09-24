//! Historical event indexing and semantic correlation subsystem.
//!
//! Provides append-only JSON Lines event logging and Jaccard token matching.

pub mod event_entry;
pub mod event_store;
pub mod similarity;

pub use event_entry::SystemEvent;
pub use event_store::EventStore;
pub use similarity::compute_jaccard_similarity;
