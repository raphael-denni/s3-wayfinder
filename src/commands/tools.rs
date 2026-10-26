// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Tools Module for s3-wayfinder CLI.
//! This module provides utility functions for the s3-wayfinder CLI application.

use anyhow::{Context, Result, ensure};

/// # Parse S3 Path
/// Parses an S3 path in the format `s3://bucket/key` and returns the bucket and key as a tuple.
///
/// Arguments:
/// - `s3_path`: A string slice representing the S3 path.
///
/// # Returns
/// A tuple containing the bucket and key as string slices.
///
/// # Errors
/// Returns an error if the path does not start with `s3://`, or if the bucket is empty.
pub fn parse_s3_path(s3_path: &str) -> Result<(&str, Option<&str>)> {
    let path = s3_path
        .strip_prefix("s3://")
        .context("S3 path must start with 's3://'")?;

    let (bucket, key) = match path.split_once('/') {
        Some((bucket, key)) => {
            let key = if key.is_empty() { None } else { Some(key) };
            (bucket, key)
        }
        None => (path, None),
    };
    ensure!(!bucket.is_empty(), "Bucket name cannot be empty.");

    Ok((bucket, key))
}
