// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Rm Command for s3-wayfinder CLI.
//! This module implements the 'rm' command for the s3-wayfinder CLI application.

use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use std::io::{self, Write};
use tracing::debug;

/// # Delete S3 Object
/// Deletes an object from the specified bucket.
pub async fn delete_object(client: &Client, bucket: &str, key: &str) -> Result<()> {
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .with_context(|| format!("Failed to delete object '{}' from bucket '{}'", key, bucket))?;

    Ok(())
}

/// # Delete S3 Bucket
/// Deletes an empty S3 bucket
pub async fn delete_bucket(client: &Client, bucket: &str) -> Result<()> {
    client
        .delete_bucket()
        .bucket(bucket)
        .send()
        .await
        .with_context(|| format!("Failed to delete bucket '{}'", bucket))?;

    Ok(())
}

/// # Run 'rm' Command
/// This function removes an S3 bucket or an object within a bucket.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `bucket`: The name of the bucket to delete or from which to delete an object.
/// - `s3_object`: An optional object key. If provided, deletes the object; otherwise, deletes the bucket.
///
/// # Errors
/// Returns an error if there are issues during the deletion operation.
pub async fn run_rm(client: &Client, bucket: &str, s3_object: Option<&str>) -> Result<()> {
    if let Some(object_key) = s3_object {
        debug!(
            "Deleting object '{}' from bucket '{}'...",
            object_key, bucket
        );

        delete_object(client, bucket, object_key).await?;

        eprintln!("Object deleted successfully.");
    } else {
        eprint!(
            "Are you sure you want to delete the bucket '{}'? (y/N): ",
            bucket
        );

        io::stderr().flush().context("Failed to flush stdout")?;

        let mut confirmation = String::new();
        io::stdin()
            .read_line(&mut confirmation)
            .context("Failed to read from stdin")?;

        if confirmation.trim().eq_ignore_ascii_case("y") {
            debug!("Deleting bucket '{}'...", bucket);

            delete_bucket(client, bucket).await?;

            eprintln!("Bucket deleted successfully.");
        } else {
            eprintln!("Bucket deletion cancelled.");
        }
    }

    Ok(())
}
