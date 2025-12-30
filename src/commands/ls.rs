// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Ls Command for sherpa-s3 CLI.
//! This module implements the 'ls' command for the sherpa-s3 CLI application.

use aws_sdk_s3::Client;

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

            match client.list_objects_v2().bucket(&bucket_name).send().await {
                Ok(output) => {
                    if let Some(contents) = output.contents {
                        if contents.is_empty() {
                            println!("No objects found in bucket '{}'.", bucket_name);
                        } else {
                            println!("Objects in '{}':", bucket_name);

                            for object in contents {
                                println!("  - {}", object.key.unwrap_or_else(|| "N/A".to_string()));
                            }
                        }
                    } else {
                        println!("No objects found in bucket '{}'.", bucket_name);
                    }
                }

                Err(e) => {
                    eprintln!(
                        "Error: Could not list objects in bucket '{}': {}",
                        bucket_name, e
                    );

                    return Err(e.into());
                }
            }
        }

        None => {
            println!("Listing S3 buckets...");

            match client.list_buckets().send().await {
                Ok(output) => {
                    if let Some(buckets) = output.buckets {
                        if buckets.is_empty() {
                            println!("No S3 buckets found.");
                        } else {
                            println!("S3 Buckets:");

                            for bucket in buckets {
                                println!(
                                    "  - {}",
                                    bucket.name.unwrap_or_else(|| "N/A".to_string())
                                );
                            }
                        }
                    } else {
                        println!("No S3 buckets found.");
                    }
                }

                Err(e) => {
                    eprintln!("Error: Could not list S3 buckets: {:?}", e);
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}
