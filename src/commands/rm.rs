// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Rm Command for s3-wayfinder CLI.
//! This module implements the 'rm' command for the s3-wayfinder CLI application.

use aws_sdk_s3::Client;
use std::io::{self, Write};

/// # Delete S3 Object
/// Deletes an object from the specified bucket.
pub async fn delete_object(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    Ok(())
}

/// # Delete S3 Bucket
/// Deletes an empty S3 bucket
pub async fn delete_bucket(
    client: &Client,
    bucket: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_bucket().bucket(bucket).send().await?;

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
pub async fn run_rm(
    client: &Client,
    bucket: &str,
    s3_object: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(object_key) = s3_object {
        println!(
            "Deleting object '{}' from bucket '{}'...",
            object_key, bucket
        );

        delete_object(client, bucket, &object_key).await?;

        println!("Object deleted successfully.");
    } else {
        println!(
            "Are you sure you want to delete the bucket '{}' and all its contents? (y/N): ",
            bucket
        );

        io::stdout().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim().eq_ignore_ascii_case("y") {
            println!("Deleting bucket '{}'...", bucket);

            delete_bucket(client, bucket).await?;

            println!("Bucket deleted successfully.");
        } else {
            println!("Bucket deletion cancelled.");
        }
    }

    Ok(())
}
