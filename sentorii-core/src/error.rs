//! Custom error types for the sentorii-core crate.

use sentorii_contracts::error::CommandBuildError;
use thiserror::Error;

/// The primary error type for operations within the sentorii-core crate.
#[derive(Debug, Error)]
pub enum CoreError {
    /// An error occurred during the serialization or deserialization of workflow state.
    #[error("State serialization error: {0}")]
    StateSerde(#[from] serde_json::Error),

    /// An I/O error occurred, typically related to reading or writing state files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to build command from step: {0}")]
    CommandBuildFailed(#[from] CommandBuildError),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("The UI failed to provide a required input")]
    InputChannelClosed,

    #[error("The event channel was closed prematurely")]
    EventChannelClosed,

    #[error("Workflow engine state is invalid: {0}")]
    InvalidState(String),
}
