//! # Sentorii Core Engine
//!
//! This crate provides the headless, message-driven engine for the Sentorii application.
//! It is responsible for receiving workflow requests, executing them against a Git repository,
//! managing persistent state, and emitting a stream of events describing its progress.

#![forbid(unsafe_code)]

mod dispatcher;
mod error;
mod git;
mod workflow;

pub use dispatcher::start_engine;
