// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Command definitions for s3-wayfinder CLI.
//! This module defines the command-line interface (CLI) commands
//! for the s3-wayfinder application using the `clap` crate.
//! It includes commands for listing, copying, and removing S3 objects.

use clap::Parser;

mod tools;

pub mod cp;
pub mod ls;
pub mod mk;
pub mod mv;
pub mod rm;

/// # CLI Commands for s3-wayfinder
///
/// # Commands
/// - `ls [bucket]`: List objects in a bucket or list all buckets if no bucket is specified.
/// - `mk`: Create a new bucket in S3 storage.
/// - `rm`: Remove an object or bucket from S3 storage.
/// - `cp` `<source>` `<destination>`: Copy an object from source to destination.
/// - `mv` `<source>` `<destination>`: Move an object from source to destination.
#[derive(Parser)]
pub enum Commands {
    /// List of objects or buckets from S3 storage
    Ls {
        /// The bucket to list objects from. If not provided, lists all buckets.
        bucket_path: Option<String>,
    },
    /// Create a new bucket in S3 storage
    Mk {
        /// The bucket path to create
        bucket_path: String,
    },
    /// Remove a bucket or object from S3 storage
    Rm {
        /// The path to the S3 bucket or object to remove. Format: s3://bucket/key
        s3_path: String,
    },
    /// Copy S3 to S3, local to S3, and S3 to local. (Objects only)
    /// Format for S3 URIs: s3://bucket/key
    Cp {
        /// The source S3 object URI or local file path
        source_path: String,
        /// The destination S3 object URI or local file path
        destination_path: String,
    },
    /// Move S3 to S3 and S3 to local. (Objects only)
    Mv {
        /// The source S3 object URI or local file path
        source_path: String,
        /// The destination S3 object URI or local file path
        destination_path: String,
    },
    /// Launch the interactive TUI for s3-wayfinder
    Tui,
}

/// # Command Line Interface for s3-wayfinder
/// This struct defines the CLI for s3-wayfinder using clap.
/// It includes subcommands for listing, copying, and removing S3 objects.
///
/// # Fields
/// - `commands`: The subcommands available in the CLI.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    /// The subcommands for the CLI.
    pub commands: Option<Commands>,
}
