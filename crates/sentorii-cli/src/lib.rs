//! # Sentorii CLI
//!
//! This crate is the official command-line interface (CLI) and Terminal User Interface (TUI)
//! for the Sentorii application. It acts as the "head" for the `sentorii-core` engine.
//!
//! Its primary responsibilities are:
//! 1.  Parsing user commands and arguments from the command line.
//! 2.  Sending `WorkflowRequest` messages to the core engine based on user input.
//! 3.  Receiving a stream of `Event` messages from the core engine.
//! 4.  Rendering a real-time, interactive TUI to visualize the progress and state
//!     of the requested workflow.

#![forbid(unsafe_code)]

pub mod app;
pub mod cli;
pub mod controller;
pub mod state;
pub mod tui;
pub mod ui;
pub mod workflow_dispatcher;

pub mod mock_engine;

pub use app::App;
