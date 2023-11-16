//! This module provides the `Args` struct which is used for argument parsing in command line applications.
//!
//! It uses the `clap` crate's `Parser` derive macro to automatically implement command line parsing,
//! and provides two options: `config_file` and `ip`.
//!
//! The `config_file` option allows specifying a custom configuration file.
//! The `ip` option, if set, prints the host's current IP to the console without performing any other actions.

use clap::{command, Parser};

/// Represents the arguments passed to the application command line.
///
/// These arguments are used to customize the behavior of the Dynamic DNS Client application:
///
/// - `config_file`: An optional argument that, if provided, specifies the path to a custom configuration file.
/// - `ip`: A flag that, when set, causes the application to print the host's current IP to the console and then exit.
/// This feature is useful for quickly checking the host's current IP without performing the usual operations of the application.
#[derive(Debug, Parser)]
#[command(author, version, long_about = "Dynamic DNS Client")]
pub struct Args {
    #[arg(short, long)]
    pub config_file: Option<String>,
    #[arg(short, long)]
    pub ip: bool,
}
