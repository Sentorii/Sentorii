//! Custom error types for the sentorii-core crate.

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
}
