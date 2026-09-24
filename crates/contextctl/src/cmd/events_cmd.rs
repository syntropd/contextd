//! Command handler for querying historical system events.

use crate::client::ContextdClient;
use anyhow::Result;
use serde_json::json;

/// Executes the `events` command.
pub async fn exec_events(
    client: &ContextdClient,
    unit: Option<&str>,
    since_seconds: u64,
    limit: usize,
    as_json: bool,
) -> Result<()> {
    let mut params = json!({
        "since_seconds": since_seconds,
        "limit": limit,
    });

    if let Some(u) = unit {
        params["unit"] = json!(u);
    }

    let res = client
        .call("io.syntrop.Context1.ListEvents", Some(params))
        .await?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    let events = res
        .get("events")
        .and_then(|e| e.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if events.is_empty() {
        println!("No system events found matching query criteria.");
        return Ok(());
    }

    println!("{:<24} {:<16} {:<24} {}", "EVENT ID", "SOURCE", "UNIT", "SUMMARY");
    println!("{}", "-".repeat(80));

    for evt in events {
        let id = evt.get("id").and_then(|v| v.as_str()).unwrap_or("-");
        let source = evt.get("source").and_then(|v| v.as_str()).unwrap_or("-");
        let unit_name = evt.get("unit").and_then(|v| v.as_str()).unwrap_or("-");
        let summary = evt.get("summary").and_then(|v| v.as_str()).unwrap_or("-");

        println!("{:<24} {:<16} {:<24} {}", id, source, unit_name, summary);
    }

    Ok(())
}
