// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Cp Command for s3-wayfinder CLI.
//! This module implements the 'cp' command for the s3-wayfinder CLI application.

use aws_sdk_s3::{Client, error::DisplayErrorContext, primitives::ByteStream};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Parses an S3 path in the format "s3://bucket/key" into a tuple of (bucket, key).
// Returns an error if the path is not valid.
fn parse_s3_path(path: &str) -> Result<(&str, &str), &'static str> {
    let path = path
        .strip_prefix("s3://")
        .ok_or("S3 path must start with 's3://'")?;

    if let Some((bucket, key)) = path.split_once('/') {
        if bucket.is_empty() || key.is_empty() {
            Err("Bucket and key cannot be empty.")
        } else {
            Ok((bucket, key))
        }
    } else {
        Err("S3 path must include a bucket and a key separated by a '/'")
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
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .copy_object()
        .copy_source(format!("{}/{}", source_bucket, source_key))
        .bucket(dest_bucket)
        .key(dest_key)
        .send()
        .await
        .map_err(|e| format!("{}", DisplayErrorContext(e)))?;
    Ok(())
}

/// # Upload Local File to S3
pub async fn upload_object(
    client: &Client,
    local_path: &str,
    dest_bucket: &str,
    dest_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file: File = File::open(local_path).await?;
    let mut contents = Vec::new();

    file.read_to_end(&mut contents).await?;

    let body = ByteStream::from(contents);

    client
        .put_object()
        .bucket(dest_bucket)
        .key(dest_key)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("{}", DisplayErrorContext(e)))?;

    Ok(())
}

/// # Download S3 Object to Local File
pub async fn download_object(
    client: &Client,
    source_bucket: &str,
    source_key: &str,
    local_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = client
        .get_object()
        .bucket(source_bucket)
        .key(source_key)
        .send()
        .await
        .map_err(|e| format!("{}", DisplayErrorContext(e)))?;

    let mut file = File::create(local_path).await?;

    while let Some(bytes) = output
        .body
        .try_next()
        .await
        .map_err(|e| format!("{}", DisplayErrorContext(e)))?
    {
        file.write_all(&bytes).await?;
    }

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
pub async fn run_cp(
    client: &Client,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
            return Err(
                "Invalid arguments: at least one of source or destination must be an S3 path."
                    .into(),
            );
        }
    }

    Ok(())
}
