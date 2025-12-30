// SPDX-License-Identifier: MIT OR Apache-2.0

//! # sherpa-s3 CLI Application
//! This is the main entry point for the sherpa-s3 CLI application.
//! It initializes the AWS S3 client, handles configuration,
//! and dispatches commands based on user input.

use aws_sdk_s3::Client;
use clap::Parser;
use tracing::info;

mod commands;
mod config;
mod tui;

// The main entry point for the sherpa-s3 CLI application.
#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    let s3_config = config::handle_config().unwrap();
    let client = Client::from_conf(s3_config);

    let cli = commands::Cli::parse();

    match &cli.commands {
        Some(commands::Commands::Ls { bucket }) => {
            info!("'ls' command called");

            if let Err(e) = commands::ls::run_ls(&client, bucket.clone()).await {
                eprintln!("Error executing ls command: {}", e);
                std::process::exit(1);
            }
        }

        Some(commands::Commands::Cp {
            source,
            destination,
        }) => {
            info!("'cp' command called");

            if let Err(e) = commands::cp::run_cp(&client, source, destination).await {
                eprintln!("Error executing cp command: {}", e);
                std::process::exit(1);
            }
        }

        Some(commands::Commands::Rm { bucket, s3_object }) => {
            info!("'rm' command called");

            if let Err(e) = commands::rm::run_rm(&client, bucket, s3_object.clone()).await {
                eprintln!("Error executing rm command: {}", e);
                std::process::exit(1);
            }
        }

        Some(commands::Commands::Tui) | None => {
            info!("'tui' interface called");

            if let Err(e) = tui::run(&client).await {
                eprintln!("Error executing the TUI: {}", e);
                std::process::exit(1);
            }
        }
    }
}
