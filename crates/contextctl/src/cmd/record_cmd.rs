//! Command handler for recording arbitrary system events into contextd.

use crate::client::ContextdClient;
use anyhow::Result;
use serde_json::json;

/// Executes the `record` command.
pub async fn exec_record(
    client: &ContextdClient,
    source: &str,
    unit: Option<&str>,
    summary: &str,
    details: Option<&str>,
) -> Result<()> {
    let mut params = json!({
        "source": source,
        "summary": summary,
    });

    if let Some(u) = unit {
        params["unit"] = json!(u);
    }
    if let Some(d) = details {
        params["details"] = json!(d);
    }

    let res = client
        .call("io.syntrop.Context1.RecordEvent", Some(params))
        .await?;

    let event_id = res
        .get("event_id")
        .and_then(|id| id.as_str())
        .unwrap_or("unknown");

    println!("Recorded event: {}", event_id);
    Ok(())
}
