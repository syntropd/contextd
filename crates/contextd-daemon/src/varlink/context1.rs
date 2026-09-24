//! Implementation of the io.syntrop.Context1 Varlink interface.
//!
//! Handles unit causality context, configuration diff inspection,
//! and system event persistence.

use super::protocol::VarlinkReply;
use contextd_core::correlator::{correlate_unit_timeline, PackageTransactionRecord};
use contextd_core::index::{EventStore, SystemEvent};
use contextd_core::watcher::DiffStore;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared state required to process Context1 method calls.
#[derive(Clone)]
pub struct Context1Handler {
    diff_store: Arc<DiffStore>,
    event_store: Arc<EventStore>,
    package_history: Arc<Vec<PackageTransactionRecord>>,
}

impl Context1Handler {
    /// Creates a new Context1Handler with the supplied storage references.
    pub fn new(
        diff_store: Arc<DiffStore>,
        event_store: Arc<EventStore>,
        package_history: Arc<Vec<PackageTransactionRecord>>,
    ) -> Self {
        Self {
            diff_store,
            event_store,
            package_history,
        }
    }

    /// Dispatches incoming io.syntrop.Context1 method calls.
    pub fn handle_call(&self, method: &str, params: Option<&Value>) -> Option<VarlinkReply> {
        match method {
            "io.syntrop.Context1.GetUnitContext" => Some(self.handle_get_unit_context(params)),
            "io.syntrop.Context1.ListRecentDiffs" => Some(self.handle_list_recent_diffs(params)),
            "io.syntrop.Context1.ListEvents" => Some(self.handle_list_events(params)),
            "io.syntrop.Context1.RecordEvent" => Some(self.handle_record_event(params)),
            _ => None,
        }
    }

    fn handle_get_unit_context(&self, params: Option<&Value>) -> VarlinkReply {
        let params = match params {
            Some(p) => p,
            None => {
                return VarlinkReply::err(
                    "io.syntrop.Context1.InvalidParameter",
                    Some(json!({ "parameter": "parameters" })),
                )
            }
        };

        let unit = match params.get("unit").and_then(|u| u.as_str()) {
            Some(u) => u,
            None => {
                return VarlinkReply::err(
                    "io.syntrop.Context1.InvalidParameter",
                    Some(json!({ "parameter": "unit" })),
                )
            }
        };

        let since_seconds = params
            .get("since_seconds")
            .and_then(|s| s.as_u64())
            .unwrap_or(3600);

        let now_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        let since_us = now_us.saturating_sub(since_seconds * 1_000_000);

        let context = correlate_unit_timeline(
            unit,
            &self.diff_store,
            &self.package_history,
            since_us,
        );

        VarlinkReply::ok(json!({ "context": context }))
    }

    fn handle_list_recent_diffs(&self, params: Option<&Value>) -> VarlinkReply {
        let since_seconds = params
            .and_then(|p| p.get("since_seconds"))
            .and_then(|s| s.as_u64())
            .unwrap_or(3600);

        let now_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        let since_us = now_us.saturating_sub(since_seconds * 1_000_000);

        match self.diff_store.list_recent_diffs(since_us) {
            Ok(diffs) => VarlinkReply::ok(json!({ "diffs": diffs })),
            Err(e) => VarlinkReply::err(
                "io.syntrop.Context1.OperationFailed",
                Some(json!({ "reason": e.to_string() })),
            ),
        }
    }

    fn handle_list_events(&self, params: Option<&Value>) -> VarlinkReply {
        let unit = params
            .and_then(|p| p.get("unit"))
            .and_then(|u| u.as_str());

        let since_seconds = params
            .and_then(|p| p.get("since_seconds"))
            .and_then(|s| s.as_u64())
            .unwrap_or(86400);

        let limit = params
            .and_then(|p| p.get("limit"))
            .and_then(|l| l.as_u64())
            .unwrap_or(100) as usize;

        let now_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        let since_us = now_us.saturating_sub(since_seconds * 1_000_000);

        match self.event_store.query(unit, since_us, limit) {
            Ok(events) => VarlinkReply::ok(json!({ "events": events })),
            Err(e) => VarlinkReply::err(
                "io.syntrop.Context1.OperationFailed",
                Some(json!({ "reason": e.to_string() })),
            ),
        }
    }

    fn handle_record_event(&self, params: Option<&Value>) -> VarlinkReply {
        let params = match params {
            Some(p) => p,
            None => {
                return VarlinkReply::err(
                    "io.syntrop.Context1.InvalidParameter",
                    Some(json!({ "parameter": "parameters" })),
                )
            }
        };

        let source = match params.get("source").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                return VarlinkReply::err(
                    "io.syntrop.Context1.InvalidParameter",
                    Some(json!({ "parameter": "source" })),
                )
            }
        };

        let summary = match params.get("summary").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                return VarlinkReply::err(
                    "io.syntrop.Context1.InvalidParameter",
                    Some(json!({ "parameter": "summary" })),
                )
            }
        };

        let unit = params.get("unit").and_then(|u| u.as_str());
        let details = params.get("details").and_then(|d| d.as_str());

        let event = SystemEvent::new(source, unit, summary, details);
        let event_id = event.id.clone();

        match self.event_store.append(&event) {
            Ok(_) => VarlinkReply::ok(json!({ "event_id": event_id })),
            Err(e) => VarlinkReply::err(
                "io.syntrop.Context1.OperationFailed",
                Some(json!({ "reason": e.to_string() })),
            ),
        }
    }
}
