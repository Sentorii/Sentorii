//! # Sentorii Configuration Crate
//!
//! This crate is the single source for all user configuration in the Sentorii project.
//! Its sole responsibility is to find, load, merge, and validate
//! configuration from multiple sources, providing a single, coherent `Config` struct
//! to the rest of the application.
//!
//! The primary entry point is the `loader::load_config` function, which provides a
//! zero-maintenance, introspective loading mechanism.
//!
//! # Example
//!
//! This example shows how to load the configuration and access a value.
//! The loader will automatically find and merge files and environment variables.
//!
//! ```no_run
//! use sentorii_config::{load_config, ConfigError};
//!
//! fn main() -> Result<(), ConfigError> {
//!     let config = load_config()?;
//!
//!     println!("Using main branch: {}", config.branching.main);
//!     println!("Using feature prefix: {}, config.branching.prefixes.feature");
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]

pub mod env;
pub mod error;
pub mod loader;
pub mod merger;
pub mod parser;
pub mod paths;
pub mod schemas;

pub use error::ConfigError;
pub use loader::load_config;
pub use schemas::Config;
