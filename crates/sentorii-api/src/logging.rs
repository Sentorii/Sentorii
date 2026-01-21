//! Provides a simple, structured logging facility for plugins.
//! All log messages are serialized as JSON and written to stderr.

use serde::{Deserialize, Serialize};

/// The severity level of a log message.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Stdout,
    Stderr,
}

/// Represents a single structured log entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    #[serde(default, skip_serializing_if = "LogSource::is_pdk")]
    pub source: LogSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum LogSource {
    #[default]
    Pdk,
    Process,
}

impl LogSource {
    #[must_use]
    pub const fn is_pdk(&self) -> bool {
        matches!(self, Self::Pdk)
    }
}
