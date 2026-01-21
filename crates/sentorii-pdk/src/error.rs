//! Defines the error handling constructs for the Sentorii PDK

use sentorii_api::{ErrorCode, ErrorResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Plugin logic failed: {0}")]
    PluginLogic(String),

    #[error("A required value was not found: {0}")]
    ValueNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Converts an internal `PdkError` into the stable serializable `ErrorResponse`
/// defined in the `sentorii-api` crate.
impl From<PdkError> for ErrorResponse {
    fn from(err: PdkError) -> Self {
        let (code, message) = match err {
            PdkError::Io(e) => (ErrorCode::IoError, e.to_string()),
            PdkError::Json(e) => (ErrorCode::JsonRequestParseError, e.to_string()),
            PdkError::PluginLogic(msg) => (ErrorCode::PluginLogicFailed, msg),
            PdkError::ValueNotFound(msg) => (ErrorCode::ValueNotFoundError, msg),
            PdkError::InvalidInput(msg) => (ErrorCode::InvalidInput, msg),
        };

        Self { code, message }
    }
}
