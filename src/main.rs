// SPDX-License-Identifier: MIT OR Apache-2.0

//! # s3-wayfinder CLI Application
//! This is the main entry point for the s3-wayfinder CLI application.
//! It initializes the AWS S3 client, handles configuration,
//! and dispatches commands based on user input.

use anyhow::Result;
use aws_sdk_s3::Client;
use clap::Parser;
use tracing::info;

mod commands;
mod config;
mod tui;

// The main entry point for the s3-wayfinder CLI application.
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    let s3_config = config::handle_config()?;
    let client = Client::from_conf(s3_config);

    let cli = commands::Cli::parse();

    match &cli.commands {
        Some(commands::Commands::Ls { bucket }) => {
            info!("'ls' command called");
            commands::ls::run_ls(&client, bucket).await?;
        }

        Some(commands::Commands::Cp {
            source,
            destination,
        }) => {
            info!("'cp' command called");
            commands::cp::run_cp(&client, source, destination).await?;
        }

        Some(commands::Commands::Rm { bucket, s3_object }) => {
            info!("'rm' command called");
            commands::rm::run_rm(&client, bucket, s3_object).await?;
        }

        Some(commands::Commands::Tui) | None => {
            info!("'tui' started");
            tui::run_tui(&client).await?;
        }
    }

    Ok(())
}
