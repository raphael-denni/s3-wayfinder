// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Cp Command for s3-wayfinder CLI.
//! This module implements the 'cp' command for the s3-wayfinder CLI application.

use anyhow::{Context, Result, bail, ensure};
use aws_sdk_s3::{Client, primitives::ByteStream};
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

// Parses an S3 path in the format "s3://bucket/key" into a tuple of (bucket, key).
// Returns an error if the path is not valid.
fn parse_s3_path(path: &str) -> Result<(&str, &str)> {
    let path = path
        .strip_prefix("s3://")
        .context("S3 path must start with 's3://'")?;

    if let Some((bucket, key)) = path.split_once('/') {
        ensure!(!bucket.is_empty(), "Bucket name cannot be empty.");
        ensure!(!key.is_empty(), "Key cannot be empty.");

        Ok((bucket, key))
    } else {
        bail!("S3 path must include a bucket and a key separated by a '/'")
    }
}

/// # Copy S3 Object
/// Copies an object from one S3 location to another
pub async fn copy_object(
    client: &Client,
    source_bucket: &str,
    source_key: &str,
    dest_bucket: &str,
    dest_key: &str,
) -> Result<()> {
    client
        .copy_object()
        .copy_source(format!("{}/{}", source_bucket, source_key))
        .bucket(dest_bucket)
        .key(dest_key)
        .send()
        .await
        .with_context(|| {
            format!(
                "Failed to copy object from s3://{}/{} to s3://{}/{}",
                source_bucket, source_key, dest_bucket, dest_key
            )
        })?;
    Ok(())
}

/// # Upload Local File to S3
pub async fn upload_object(
    client: &Client,
    local_path: &str,
    dest_bucket: &str,
    dest_key: &str,
) -> Result<()> {
    let body = ByteStream::from_path(local_path)
        .await
        .with_context(|| format!("Failed to read local file '{}'", local_path))?;

    client
        .put_object()
        .bucket(dest_bucket)
        .key(dest_key)
        .body(body)
        .send()
        .await
        .with_context(|| {
            format!(
                "Failed to upload file '{}' to s3://{}/{}",
                local_path, dest_bucket, dest_key
            )
        })?;

    Ok(())
}

/// # Download S3 Object to Local File
pub async fn download_object(
    client: &Client,
    source_bucket: &str,
    source_key: &str,
    local_path: &str,
) -> Result<()> {
    let mut output = client
        .get_object()
        .bucket(source_bucket)
        .key(source_key)
        .send()
        .await
        .with_context(|| {
            format!(
                "Failed to download object s3://{}/{} to '{}'",
                source_bucket, source_key, local_path
            )
        })?;

    let file = File::create(local_path)
        .await
        .with_context(|| format!("Failed to create local file '{}'", local_path))?;
    let mut writer = BufWriter::new(file);

    while let Some(bytes) = output
        .body
        .try_next()
        .await
        .context("Error reading data from S3 stream")?
    {
        writer
            .write_all(&bytes)
            .await
            .context("Failed to write data to local file")?;
    }

    writer.flush().await.context("Failed to flush local file")?;

    Ok(())
}

/// # Run 'cp' Command
/// This function copies an S3 object from the specified source to the destination.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `source`: The source S3 object URI or local file path.
/// - `destination`: The destination S3 object URI or local file path.
///
/// # Errors
/// Returns an error if there are issues during the copy operation.
pub async fn run_cp(client: &Client, source: &str, destination: &str) -> Result<()> {
    let source_is_s3 = source.starts_with("s3://");
    let destination_is_s3 = destination.starts_with("s3://");

    match (source_is_s3, destination_is_s3) {
        (true, true) => {
            let (source_bucket, source_key) = parse_s3_path(source)?;
            let (dest_bucket, dest_key) = parse_s3_path(destination)?;

            println!("Copying from '{}' to '{}'", source, destination);

            copy_object(client, source_bucket, source_key, dest_bucket, dest_key).await?;

            println!(
                "Successfully copied object from '{}' to '{}'",
                source, destination
            );
        }

        (false, true) => {
            let (dest_bucket, dest_key) = parse_s3_path(destination)?;

            println!(
                "Uploading from local '{}' to S3 '{}'...",
                source, destination,
            );

            upload_object(client, source, dest_bucket, dest_key).await?;

            println!(
                "Successfully uploaded local file '{}' to S3 '{}'",
                source, destination
            );
        }

        (true, false) => {
            let (source_bucket, source_key) = parse_s3_path(source)?;

            println!(
                "Downloading from S3 '{}' to local '{}'...",
                source, destination,
            );

            download_object(client, source_bucket, source_key, destination).await?;

            println!(
                "Successfully downloaded S3 '{}' to local '{}'",
                source, destination
            );
        }

        (false, false) => {
            anyhow::bail!(
                "Invalid arguments: at least one of source or destination must be an S3 path."
            );
        }
    }

    Ok(())
}
