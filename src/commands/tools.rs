// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Tools Module for s3-wayfinder CLI.
//! This module provides utility functions for the s3-wayfinder CLI application.

use anyhow::{Context, Result, bail, ensure};

/// # Parse S3 Path
/// Parses an S3 path in the format `s3://bucket/key` and returns the bucket and key as a tuple.
///
/// Arguments:
/// - `path`: A string slice representing the S3 path.
///
/// # Returns
/// A tuple containing the bucket and key as string slices.
///
/// # Errors
/// Returns an error if the path does not start with `s3://`, or if the bucket or key is empty.
pub fn parse_s3_path(path: &str) -> Result<(&str, &str)> {
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
