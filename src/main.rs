// SPDX-License-Identifier: MIT OR Apache-2.0

//! # s3-wayfinder CLI/TUI Application
//! This is the main entry point for the s3-wayfinder CLI/TUI application.
//! It initializes the AWS S3 client, handles configuration,
//! and dispatches commands based on user input.

use anyhow::Result;
use aws_sdk_s3::Client;
use clap::Parser;
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

mod commands;
mod config;
mod tui;

// Command dispatcher that routes the CLI commands to their respective handlers.
async fn execute_command(client: &Client, cli: commands::Cli) -> Result<()> {
    match &cli.commands {
        Some(commands::Commands::Ls { bucket_path }) => {
            debug!("'ls' command called");
            commands::ls::run(client, bucket_path.as_deref()).await?;
        }

        Some(commands::Commands::Mk { bucket_path }) => {
            debug!("'mk' command called");
            commands::mk::run(client, bucket_path).await?;
        }

        Some(commands::Commands::Rm { s3_path }) => {
            debug!("'rm' command called");
            commands::rm::run(client, s3_path).await?;
        }

        Some(commands::Commands::Cp {
            source_path,
            destination_path,
        }) => {
            debug!("'cp' command called");
            commands::cp::run(client, source_path, destination_path).await?;
        }

        Some(commands::Commands::Mv {
            source_path,
            destination_path,
        }) => {
            debug!("'mv' command called");
            commands::mv::run(client, source_path, destination_path).await?;
        }

        Some(commands::Commands::Tui) | None => {
            debug!("'tui' started");
            tui::run(client).await?;
        }
    }

    Ok(())
}

// The main entry point for the s3-wayfinder CLI application.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = commands::Cli::parse();
    let result = match config::load(cli.profile.as_deref()) {
        Ok(config) => execute_command(&Client::from_conf(config), cli).await,
        Err(err) => Err(err),
    };

    if let Err(err) = result {
        error!(error = ?err, "Application crashed with");
        std::process::exit(1);
    }

    Ok(())
}
