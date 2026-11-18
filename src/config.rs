// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Configuration management for s3-wayfinder.
//! This module handles loading, saving, and managing the configuration
//! file that contains S3 credentials and settings.
//!
//! The configuration file is stored in the user's configuration directory
//! under a subdirectory named `s3-wayfinder`.

use anyhow::{Context, Result, bail};
use aws_sdk_s3::config::{Credentials, Region};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::ErrorKind;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
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
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub endpoint_url: Option<String>,
}

const DEFAULT_CONFIG_PATH: &str = "s3-wayfinder/config.toml";

// Determines the platform-specific configuration file path and returns it as a PathBuf.
fn get_config_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not find config directory")?;
    path.push(DEFAULT_CONFIG_PATH);
    Ok(path)
}

// Read and deserialize the TOML configuration file into a Config struct.
// Return an error if the file does not exist or cannot be read/parsed.
fn load_from_path(config_path: &PathBuf, profile_name: Option<&str>) -> Result<Config> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file at {:?}", config_path))?;
    let config_table = toml::Table::from_str(&content)
        .with_context(|| format!("Failed to parse config file at {:?}", config_path))?;

    let profile_name =
        profile_name.or_else(|| config_table.get("default_profile").and_then(|v| v.as_str()));

    let profile_config = match profile_name {
        Some(profile_name) => {
            if let Some(profile_table) = config_table
                .get("profiles")
                .and_then(|p| p.get(profile_name))
            {
                profile_table
                    .clone()
                    .try_into::<Config>()
                    .with_context(|| {
                        format!(
                            "Failed to deserialize profile '{}' in config file at {:?}",
                            profile_name, config_path
                        )
                    })?
            } else {
                bail!(
                    "Profile '{}' not found in config file at {:?}",
                    profile_name,
                    config_path
                );
            }
        }
        None => {
            bail!(
                "No profile specified and default_profile not set in config file at {:?}",
                config_path
            );
        }
    };

    debug!(
        "Loaded profile configuration {} from {:?}",
        profile_name.unwrap_or_default(),
        config_path
    );
    Ok(profile_config)
}

// Create a new configuration file with default values.
// Returns an error if the file cannot be written.
fn create_config_file(config_path: &PathBuf) -> Result<()> {
    debug!("No configuration file found. Creating a default one.");

    let default_config = Config {
        access_key_id: "YOUR_ACCESS_KEY_ID".to_string(),
        secret_access_key: "YOUR_SECRET_ACCESS_KEY".to_string(),
        region: "us-east-1".to_string(),
        endpoint_url: None,
    };
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory at {:?}", parent))?;
    }

    let mut content = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default configuration")?;
    content = format!(
        "{}{}{}{}{}",
        "# Configuration file for s3-wayfinder. Fill in your S3 credentials and settings below before using it.\n",
        "default_profile = \"default\"\n\n",
        "[profiles.default]\n",
        content,
        "# endpoint_url = \"\"\n"
    );

    fs::write(config_path, content)
        .with_context(|| format!("Failed to write config file to {:?}", config_path))?;
    #[cfg(unix)]
    fs::set_permissions(config_path, fs::Permissions::from_mode(0o600)).with_context(|| {
        format!(
            "Failed to set permissions \"600\" for config file at {:?}",
            config_path
        )
    })?;

    debug!("Configuration saved to {:?}", config_path);
    Ok(())
}

// Open the app config file inside the user defined editor or system default editor.
fn open_in_editor(config_path: &PathBuf) -> Result<()> {
    let default_editor = if cfg!(windows) { "notepad" } else { "nano" }.to_string();
    let editor_path = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .ok()
        .unwrap_or(default_editor);
    let mut parts = editor_path.split_whitespace();
    let prog = parts
        .next()
        .context("Fatal: Could not find a suitable editor to edit config")?;

    debug!(
        "Starting editor to edit the configuration file at {:?}",
        config_path
    );
    process::Command::new(prog)
        .args(parts)
        .arg(config_path)
        .status()?;
    Ok(())
}

pub fn edit() -> Result<()> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        create_config_file(&config_path)?;
    }
    open_in_editor(&config_path)
}

/// # Load Configuration
/// Load the configuration file, create a default one if it doesn't exist,
/// and then setup the AWS S3 configuration.
///
/// # Errors
/// Returns an error if there are issues loading or saving the configuration file.
///
/// # Returns
/// Ok(aws_sdk_s3::Config) - The configured AWS S3 configuration object.
pub fn load(profile_name: Option<&str>) -> Result<aws_sdk_s3::Config> {
    let config_path = get_config_path()?;
    let config = match load_from_path(&config_path, profile_name) {
        Ok(config) => config,
        Err(err) => {
            if !err
                .downcast_ref::<std::io::Error>()
                .is_some_and(|io_err| io_err.kind() == ErrorKind::NotFound)
            {
                return Err(err).context("Fatal: Error loading configuration");
            }
            create_config_file(&config_path).context("Fatal: Error during first-time setup")?;
            open_in_editor(&config_path)?;

            load_from_path(&config_path, profile_name)
                .context("Fatal: Error loading new configuration")?
        }
    };

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

    Ok(s3_config_builder.build())
}
