//! Command handler for listing recent system configuration diffs.

use crate::client::ContextdClient;
use anyhow::Result;
use serde_json::json;

/// Executes the `diffs` command.
pub async fn exec_diffs(client: &ContextdClient, since_seconds: u64, as_json: bool) -> Result<()> {
    let params = json!({
        "since_seconds": since_seconds,
    });

    let res = client
        .call("io.syntrop.Context1.ListRecentDiffs", Some(params))
        .await?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    let diffs = res
        .get("diffs")
        .and_then(|d| d.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if diffs.is_empty() {
        println!("No configuration diffs recorded within the past {} seconds.", since_seconds);
        return Ok(());
    }

    println!("Recent Configuration Diffs ({})", diffs.len());
    for diff in diffs {
        let path = diff.get("file_path").and_then(|p| p.as_str()).unwrap_or("unknown");
        let ts = diff.get("timestamp_us").and_then(|t| t.as_u64()).unwrap_or(0);
        println!("\n--- {} (timestamp: {}) ---", path, ts);
        if let Some(content) = diff.get("diff_content").and_then(|c| c.as_str()) {
            println!("{}", content.trim_end());
        }
    }

    Ok(())
}
