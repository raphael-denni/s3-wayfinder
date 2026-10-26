// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Mv Command for s3-wayfinder CLI.
//! This module implements the 'mv' command for the s3-wayfinder CLI application.

use crate::commands::{cp, rm, tools};
use anyhow::{Result, bail, ensure};
use aws_sdk_s3::Client;

/// # Run 'mv' Command
/// This function moves an object from a source to a destination in S3.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `source_path`: The source S3 object path.
/// - `destination_path`: The destination S3 object path or local file path.
///
/// # Errors
/// Returns an error if there are issues during the move operation.
pub async fn run_mv(client: &Client, source_path: &str, destination_path: &str) -> Result<()> {
    let dest_is_s3_path = destination_path.starts_with("s3://");
    let (source_bucket, source_key) = match tools::parse_s3_path(source_path) {
        Ok((bucket, key)) => (bucket, key),
        Err(e) => {
            bail!("Failed to parse source path '{}': {}", source_path, e);
        }
    };
    ensure!(
        source_key.is_some(),
        "Source path '{}' must include an object key.",
        source_path
    );

    if dest_is_s3_path {
        let (dest_bucket, dest_key) = match tools::parse_s3_path(destination_path) {
            Ok((bucket, key)) => (bucket, key),
            Err(e) => {
                bail!(
                    "Failed to parse destination path '{}': {}",
                    destination_path,
                    e
                );
            }
        };
        ensure!(
            dest_key.is_some(),
            "Destination path '{}' must include an object key.",
            destination_path
        );

        cp::copy_object(
            client,
            source_bucket,
            source_key.unwrap(),
            dest_bucket,
            dest_key.unwrap(),
        )
        .await?;
    } else {
        cp::download_object(client, source_bucket, source_key.unwrap(), destination_path).await?;
    }
    rm::delete_object(client, source_bucket, source_key.unwrap()).await?;

    eprintln!(
        "Successfully moved object from '{}' to '{}'.",
        source_path, destination_path
    );
    Ok(())
}
