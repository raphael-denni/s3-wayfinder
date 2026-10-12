// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Mv Command for s3-wayfinder CLI.
//! This module implements the 'mv' command for the s3-wayfinder CLI application.

use crate::commands::{cp, rm, tools};
use anyhow::{Result, bail};
use aws_sdk_s3::Client;
use gag::Gag;

/// # Run 'mv' Command
/// This function moves an object from a source to a destination in S3.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `source`: The source S3 object key.
/// - `destination`: The destination S3 object key.
///
/// # Errors
/// Returns an error if there are issues during the move operation.
pub async fn run_mv(client: &Client, source: &str, destination: &str) -> Result<()> {
    let (source_bucket, source_key) = match tools::parse_s3_path(source) {
        Ok((bucket, key)) => (bucket, key),
        Err(e) => {
            bail!("Failed to parse source path '{}': {}", source, e);
        }
    };

    {
        let _gag = Gag::stderr().unwrap();
        cp::run_cp(client, source, destination).await?;
        rm::run_rm(client, source_bucket, Some(source_key)).await?;
    }

    eprintln!(
        "Successfully moved object from '{}' to '{}'.",
        source, destination
    );
    Ok(())
}
