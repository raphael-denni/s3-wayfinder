// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Configuration management for s3-wayfinder.
//! This module handles loading, saving, and managing the configuration
//! file that contains S3 credentials and settings.
//!
//! The configuration file is stored in the user's configuration directory
//! under a subdirectory named `s3-wayfinder`.

use aws_sdk_s3::{config::Credentials, config::Region};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

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
fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let app_config_dir = config_dir.join("s3-wayfinder");

    fs::create_dir_all(&app_config_dir)?;

    Ok(app_config_dir.join("config.toml"))
}

// Read and deserialize the TOML configuration file into a Config struct.
// Return an error if the file does not exist or cannot be read/parsed.
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err("Configuration file does not exist. Please create one".into());
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

// Saves the Config struct to the TOML configuration file.
// Returns an error if the file cannot be written.
fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)?;

    fs::write(&config_path, content)?;

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
pub fn handle_config() -> Result<aws_sdk_s3::Config, Box<dyn std::error::Error>> {
    let config = match load_config() {
        Ok(config) => {
            info!("Successfully loaded configuration.");
            config
        }

        Err(e) if e.to_string().contains("does not exist") => {
            info!("No configuration file found. Creating a default one.");

            let new_config = Config {
                access_key_id: "YOUR_ACCESS_KEY_ID".to_string(),
                secret_access_key: "YOUR_SECRET_ACCESS_KEY".to_string(),
                region: "us-east-1".to_string(),
                endpoint_url: None,
            };

            if let Err(save_err) = save_config(&new_config) {
                eprintln!(
                    "Fatal: Could not save the new configuration file: {}",
                    save_err
                );

                return Err(save_err.into());
            }

            println!("Default configuration file created at the standard location.");
            println!("Please edit it with your S3 credentials before proceeding.");

            println!(
                "For S3-compatible services, add and uncomment the `endpoint_url` key in your config file. For example:"
            );
            println!("# endpoint_url = \"s3.us-east-001.compatible-service.com\"");

            new_config
        }

        Err(e) => {
            eprintln!("Fatal: Error loading configuration: {}", e);
            return Err(e);
        }
    };

    info!("Current configuration: {:?}", config);

    let credentials = Credentials::new(
        &config.access_key_id,
        &config.secret_access_key,
        None,
        None,
        "Static",
    );

    let region = Region::new(config.region.clone());

    let mut s3_config_builder = aws_sdk_s3::Config::builder()
        .credentials_provider(credentials)
        .region(region);

    if let Some(endpoint_url) = &config.endpoint_url {
        s3_config_builder = s3_config_builder.endpoint_url(endpoint_url);
    }

    let s3_config = s3_config_builder.build();

    Ok(s3_config)
}
