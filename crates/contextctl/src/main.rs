//! Main binary entry point for contextctl CLI.

use anyhow::Result;
use clap::{CommandFactory, Parser};
use contextctl::cli::{Cli, Commands};
use contextctl::client::ContextdClient;
use contextctl::cmd::*;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = ContextdClient::new(&cli.socket);

    match cli.command {
        Commands::Unit { unit, since } => {
            exec_unit(&client, &unit, since, cli.json).await?;
        }
        Commands::Diffs { since } => {
            exec_diffs(&client, since, cli.json).await?;
        }
        Commands::Events { unit, since, limit } => {
            exec_events(&client, unit.as_deref(), since, limit, cli.json).await?;
        }
        Commands::Record {
            source,
            unit,
            summary,
            details,
        } => {
            exec_record(
                &client,
                &source,
                unit.as_deref(),
                &summary,
                details.as_deref(),
            )
            .await?;
        }
        Commands::Info => {
            exec_info(&client, cli.json).await?;
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            exec_completions(&mut cmd, shell);
        }
    }

    Ok(())
}
