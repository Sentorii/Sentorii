//! This module defines the complete command-line interface for the `sentorii-cli` application.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Feature(Feature),
}

#[derive(Parser, Debug)]
pub struct Feature {
    #[command(subcommand)]
    pub command: FeatureCommands,
}

#[derive(Subcommand, Debug)]
pub enum FeatureCommands {
    Start,
}