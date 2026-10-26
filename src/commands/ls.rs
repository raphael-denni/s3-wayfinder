// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Ls Command for s3-wayfinder CLI.
//! This module implements the 'ls' command for the s3-wayfinder CLI application.

use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use tracing::debug;

/// # List S3 Buckets
/// Returns a list of bucket names
pub async fn list_buckets(client: &Client) -> Result<Vec<String>> {
    let output = client
        .list_buckets()
        .send()
        .await
        .context("Failed to list S3 buckets")?;

    let buckets = output
        .buckets
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.name.unwrap_or_else(|| "N/A".to_string()))
        .collect();

    Ok(buckets)
}

/// # List S3 Objects
/// Returns a list of object keys in the specified bucket.
pub async fn list_objects(client: &Client, bucket: &str) -> Result<Vec<String>> {
    let mut paginator = client
        .list_objects_v2()
        .bucket(bucket)
        .into_paginator()
        .send();
    let mut objects = Vec::new();

    while let Some(page) = paginator.next().await {
        let output =
            page.with_context(|| format!("Failed to list objects in bucket '{}'", bucket))?;

        let page_objects = output
            .contents
            .unwrap_or_default()
            .into_iter()
            .map(|o| o.key.unwrap_or_else(|| "N/A".to_string()));

        objects.extend(page_objects);
    }

    Ok(objects)
}

/// # Run 'ls' Command
/// This function lists S3 buckets or objects within a specified bucket.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `bucket_path`: An optional bucket path. If provided, lists objects in that bucket; otherwise, lists all buckets.
///
/// # Errors
/// Returns an error if there are issues listing buckets or objects.
pub async fn run_ls(client: &Client, bucket_path: Option<&str>) -> Result<()> {
    match bucket_path {
        Some(bucket_path) => {
            let bucket_name = bucket_path
                .strip_prefix("s3://")
                .context("Bucket path must start with 's3://'")?;
            debug!("Listing objects in bucket: {}.", bucket_name);

            let objects = list_objects(client, bucket_name).await?;

            if objects.is_empty() {
                eprintln!("No objects found in bucket '{}'.", bucket_name);
            } else {
                eprintln!("Found object(s): ");

                for object in objects {
                    println!("{}", object);
                }
            }
        }

        None => {
            let buckets = list_buckets(client).await?;

            if buckets.is_empty() {
                eprintln!("No S3 buckets found.");
            } else {
                eprintln!("Found bucket(s): ");

                for bucket in buckets {
                    println!("{}", bucket);
                }
            }
        }
    }

    Ok(())
}
