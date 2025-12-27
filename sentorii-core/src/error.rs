//! Custom error types for the sentorii-core crate.

use sentorii_config::ConfigError;
use sentorii_contracts::error::{CommandBuildError, CommandExecutionError};
use thiserror::Error;
use tokio::task::JoinError;

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
    CommandExecutionFailed(#[from] CommandExecutionError),

    #[error("Workflow engine state is invalid: {0}")]
    InvalidState(#[from] InvalidStateError),

    #[error("Config error: {0}")]
    ConfigError(#[from] ConfigError),
}

/// An error related to the engine's internal state or communication channels.
#[derive(Debug, Error)]
pub enum InvalidStateError {
    #[error("The event channel was closed unexpectedly. The UI may have disconnected.")]
    EventChannelClosed,
    #[error("The UI failed to provide a required input; the response channel was closed.")]
    InputChannelClosed,
    #[error("Attempted to operate on a workflow that is not in a runnable state (status: {0})")]
    NotRunnable(String),
    #[error("The output from a critical git command was not valid UTF-8 text.")]
    InvalidGitOutput,
}

impl From<JoinError> for CoreError {
    fn from(_: JoinError) -> Self {
        Self::CommandExecutionFailed(CommandExecutionError::TaskPanic)
    }
}
