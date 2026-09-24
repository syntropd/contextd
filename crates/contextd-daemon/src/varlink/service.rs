//! Standard org.varlink.service introspection for contextd.

use super::protocol::VarlinkReply;
use serde_json::json;

/// Varlink interface definition text for io.syntrop.Context1.
pub const IO_SYNTROP_CONTEXT1_INTERFACE: &str = r#"
interface io.syntrop.Context1

type ConfigDiff (
  file_path: string,
  timestamp_us: int,
  diff_content: string
)

type PackageTransaction (
  timestamp_us: int,
  action: string,
  package_name: string,
  version: string
)

type UnitContext (
  unit_name: string,
  config_diffs: []ConfigDiff,
  package_upgrades: []PackageTransaction,
  summary: string
)

type Event (
  id: string,
  timestamp_us: int,
  source: string,
  unit: ?string,
  summary: string,
  details: ?string
)

method GetUnitContext(unit: string, since_seconds: int) -> (context: UnitContext)
method ListRecentDiffs(since_seconds: int) -> (diffs: []ConfigDiff)
method ListEvents(unit: ?string, since_seconds: int, limit: int) -> (events: []Event)
method RecordEvent(source: string, unit: ?string, summary: string, details: ?string) -> (event_id: string)

error InvalidParameter(parameter: string)
error OperationFailed(reason: string)
"#;

/// Handles standard org.varlink.service method dispatches.
pub fn handle_service_call(method: &str, params: Option<&serde_json::Value>) -> Option<VarlinkReply> {
    match method {
        "org.varlink.service.GetInfo" => Some(VarlinkReply::ok(json!({
            "vendor": "Syntropd Project",
            "product": "contextd",
            "version": "0.1.0",
            "url": "https://github.com/syntropd/contextd",
            "interfaces": [
                "org.varlink.service",
                "io.syntrop.Context1"
            ]
        }))),
        "org.varlink.service.GetInterfaceDescription" => {
            let iface = params
                .and_then(|p| p.get("interface"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match iface {
                "io.syntrop.Context1" => Some(VarlinkReply::ok(json!({
                    "description": IO_SYNTROP_CONTEXT1_INTERFACE.trim()
                }))),
                _ => Some(VarlinkReply::err(
                    "org.varlink.service.InterfaceNotFound",
                    Some(json!({ "interface": iface })),
                )),
            }
        }
        _ => None,
    }
}
