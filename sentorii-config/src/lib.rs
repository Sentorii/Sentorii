#![forbid(unsafe_code)]

//! # Sentorii Config
//! 
//! This crate is the single source of truth for all user configuration.
//! Its sole responsibility is to find, load, parse, merge, and validate
//! `sentorii.toml` files, providing a single, coherent `Config` struct
//! to the rest of the application.

pub mod error;
pub mod loader;
pub mod schemas;
