// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Mk Command for s3-wayfinder CLI.
//! This module implements the 'mk' command for the s3-wayfinder CLI application.

use anyhow::{Context, Result};

/// # Run 'mk' Command
/// This function creates a new S3 bucket.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `bucket`: The name of the S3 bucket to create.
///
/// # Errors
/// Returns an error if there are issues during the bucket creation operation.
pub async fn run_mk(client: &aws_sdk_s3::Client, bucket: &str) -> Result<()> {
    client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .with_context(|| format!("Failed to create bucket '{}'", bucket))?;

    eprintln!("Bucket '{}' created successfully.", bucket);
    Ok(())
}
