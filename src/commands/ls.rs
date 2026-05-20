// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Ls Command for sherpa-s3 CLI.
//! This module implements the 'ls' command for the sherpa-s3 CLI application.

use aws_sdk_s3::Client;

/// # List S3 Buckets
/// Returns a list of bucket names
pub async fn list_buckets(client: &Client) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = client.list_buckets().send().await?;

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
pub async fn list_objects(
    client: &Client,
    bucket: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = client.list_objects_v2().bucket(bucket).send().await?;

    let objects = output
        .contents
        .unwrap_or_default()
        .into_iter()
        .map(|o| o.key.unwrap_or_else(|| "N/A".to_string()))
        .collect();

    Ok(objects)
}

/// # Run 'ls' Command
/// This function lists S3 buckets or objects within a specified bucket.
///
/// # Arguments
/// - `client`: An instance of the AWS S3 client.
/// - `bucket`: An optional bucket name. If provided, lists objects in that bucket; otherwise, lists all buckets.
///
/// # Errors
/// Returns an error if there are issues listing buckets or objects.
pub async fn run_ls(
    client: &Client,
    bucket: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match bucket {
        Some(bucket_name) => {
            println!("Listing objects in bucket: {}", bucket_name);

            let objects = list_objects(client, &bucket_name).await?;

            if objects.is_empty() {
                println!("No objects found in bucket '{}'.", bucket_name);
            } else {
                println!("Objects in '{}':", bucket_name);

                for object in objects {
                    println!("  - {}", object);
                }
            }
        }

        None => {
            println!("Listing S3 buckets...");

            let buckets = list_buckets(client).await?;

            if buckets.is_empty() {
                println!("No S3 buckets found.");
            } else {
                println!("S3 Buckets:");

                for bucket in buckets {
                    println!("  - {}", bucket);
                }
            }
        }
    }

    Ok(())
}
