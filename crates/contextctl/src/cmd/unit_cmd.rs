//! Command handler for inspecting unit causality context.

use crate::client::ContextdClient;
use anyhow::Result;
use serde_json::json;

/// Executes the `unit` command.
pub async fn exec_unit(
    client: &ContextdClient,
    unit: &str,
    since_seconds: u64,
    as_json: bool,
) -> Result<()> {
    let params = json!({
        "unit": unit,
        "since_seconds": since_seconds,
    });

    let res = client
        .call("io.syntrop.Context1.GetUnitContext", Some(params))
        .await?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    let context = res.get("context").unwrap_or(&res);
    let summary = context
        .get("summary")
        .and_then(|s| s.as_str())
        .unwrap_or("No summary provided");

    println!("Unit Context: {}", unit);
    println!("Summary: {}", summary);
    println!();

    if let Some(diffs) = context.get("config_diffs").and_then(|d| d.as_array()) {
        println!("Configuration Diffs ({}):", diffs.len());
        for diff in diffs {
            let path = diff.get("file_path").and_then(|p| p.as_str()).unwrap_or("unknown");
            let ts = diff.get("timestamp_us").and_then(|t| t.as_u64()).unwrap_or(0);
            println!("  * [{}] {}", ts, path);
            if let Some(content) = diff.get("diff_content").and_then(|c| c.as_str()) {
                for line in content.lines() {
                    println!("    {}", line);
                }
            }
        }
    }

    if let Some(pkgs) = context.get("package_upgrades").and_then(|p| p.as_array()) {
        println!();
        println!("Package Upgrades ({}):", pkgs.len());
        for pkg in pkgs {
            let name = pkg.get("package_name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let action = pkg.get("action").and_then(|a| a.as_str()).unwrap_or("unknown");
            let ver = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
            println!("  * {} {} ({})", action, name, ver);
        }
    }

    Ok(())
}
