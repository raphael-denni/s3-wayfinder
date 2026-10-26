// SPDX-License-Identifier: MIT OR Apache-2.0

//! # s3-wayfinder CLI Application
//! This is the main entry point for the s3-wayfinder CLI application.
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

// The main entry point for the s3-wayfinder CLI application.
async fn run_app() -> Result<()> {
    let s3_config = config::handle_config()?;
    let client = Client::from_conf(s3_config);

    let cli = commands::Cli::parse();
    match &cli.commands {
        Some(commands::Commands::Ls { bucket_path }) => {
            debug!("'ls' command called");
            commands::ls::run_ls(&client, bucket_path.as_deref()).await?;
        }

        Some(commands::Commands::Mk { bucket_path }) => {
            debug!("'mk' command called");
            commands::mk::run_mk(&client, bucket_path).await?;
        }

        Some(commands::Commands::Rm { s3_path }) => {
            debug!("'rm' command called");
            commands::rm::run_rm(&client, s3_path).await?;
        }

        Some(commands::Commands::Cp {
            source_path,
            destination_path,
        }) => {
            debug!("'cp' command called");
            commands::cp::run_cp(&client, source_path, destination_path).await?;
        }

        Some(commands::Commands::Mv {
            source_path,
            destination_path,
        }) => {
            debug!("'mv' command called");
            commands::mv::run_mv(&client, source_path, destination_path).await?;
        }

        Some(commands::Commands::Tui) | None => {
            debug!("'tui' started");
            tui::run_tui(&client).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    if let Err(err) = run_app().await {
        error!(error = ?err, "Application crashed with");
        // eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }

    Ok(())
}
