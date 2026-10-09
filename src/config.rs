// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Configuration management for s3-wayfinder.
//! This module handles loading, saving, and managing the configuration
//! file that contains S3 credentials and settings.
//!
//! The configuration file is stored in the user's configuration directory
//! under a subdirectory named `s3-wayfinder`.

use anyhow::{Context, Result};
use aws_sdk_s3::{config::Credentials, config::Region};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use tracing::debug;

/// # Configuration Structure
/// Represents the configuration settings for S3 access.
///
/// # Fields
/// - `access_key_id`: The AWS access key ID.
/// - `secret_access_key`: The AWS secret access key.
/// - `region`: The AWS region.
/// - `endpoint_url`: An optional custom endpoint URL for S3-compatible services.
///
/// # Derives
/// - `Debug`: For formatting the configuration for debugging purposes.
/// - `Deserialize`: For deserializing the configuration from TOML format.
/// - `Serialize`: For serializing the configuration to TOML format.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint_url: Option<String>,
}

// Determines the platform-specific configuration file path and ensures the directory exists.
// Returns a PathBuf to the configuration file.
fn get_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;

    path.push("s3-wayfinder");
    path.push("config.toml");

    Ok(path)
}

// Read and deserialize the TOML configuration file into a Config struct.
// Return an error if the file does not exist or cannot be read/parsed.
fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;
    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {:?}", config_path))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file at {:?}", config_path))?;

    debug!("Loaded configuration from {:?}", config_path);

    Ok(config)
}

// Saves the Config struct to the TOML configuration file.
// Returns an error if the file cannot be written.
fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
    }

    let content = toml::to_string_pretty(config).context("Failed to serialize configuration")?;
    fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file to {:?}", config_path))?;

    debug!("Configuration saved to {:?}", config_path);

    Ok(())
}

/// # Handle Configuration
/// Handles loading the configuration file, creating a default one if it doesn't exist,
/// and setting up the AWS S3 configuration.
///
/// # Errors
/// Returns an error if there are issues loading or saving the configuration file.
///
/// # Returns
/// Ok(aws_sdk_s3::Config) - The configured AWS S3 configuration object.
pub fn handle_config() -> Result<aws_sdk_s3::Config> {
    let config = match load_config() {
        Ok(config) => {
            debug!("Successfully loaded configuration.");
            config
        }

        Err(e) => {
            if e.downcast_ref::<std::io::Error>()
                .is_some_and(|io| io.kind() == ErrorKind::NotFound)
            {
                debug!("No configuration file found. Creating a default one.");

                let new_config = Config {
                    access_key_id: "YOUR_ACCESS_KEY_ID".to_string(),
                    secret_access_key: "YOUR_SECRET_ACCESS_KEY".to_string(),
                    region: "us-east-1".to_string(),
                    endpoint_url: None,
                };

                save_config(&new_config)
                    .context("Fatal: Could not save a new configuration file")?;

                eprintln!(
                    "Default configuration file created at standard location,\n
                     please edit it with your S3 credentials before proceeding."
                );

                new_config
            } else {
                return Err(e).context("Fatal: Error loading configuration");
            }
        }
    };

    let debug_config = Config {
        access_key_id: "REDACTED".to_string(),
        secret_access_key: "REDACTED".to_string(),
        region: config.region.clone(),
        endpoint_url: config.endpoint_url.clone(),
    };
    debug!("Current configuration: {:?}", debug_config);

    let Config {
        access_key_id,
        secret_access_key,
        region,
        endpoint_url,
    } = config;

    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "Static");
    let region = Region::new(region);

    let mut s3_config_builder = aws_sdk_s3::Config::builder()
        .credentials_provider(credentials)
        .region(region);

    if let Some(url) = endpoint_url {
        let url = if url.starts_with("http") {
            url
        } else if url.starts_with("localhost") || url.starts_with("127.0.0.1") {
            format!("http://{}", url)
        } else {
            format!("https://{}", url)
        };

        s3_config_builder = s3_config_builder.endpoint_url(url).force_path_style(true);
    }

    let s3_config = s3_config_builder.build();

    Ok(s3_config)
}
