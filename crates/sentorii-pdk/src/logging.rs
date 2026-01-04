//! Provides a simple, structured logging facility for plugins.
//! All log messages are serialized as JSON and written to stderr.

use serde::Serialize;
use std::io::{self, Write};

/// The severity level of a log message.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Represents a single structured log entry.
#[derive(Serialize, Debug)]
struct LogEntry<'a> {
    level: LogLevel,
    message: &'a str,
}

/// Writes a log entry to stderr with the given level and message.
///
/// This function locks stderr, serializes the log entry to JSON, and writes it,
/// followed by a newline. It is the core logging primitive used by the public
/// logging functions.
fn log(level: LogLevel, message: &str) {
    let entry = LogEntry { level, message };
    if let Ok(json) = serde_json::to_string(&entry) {
        let mut stderr = io::stderr().lock();
        let _ = writeln!(stderr, "{}", json);
    }
}

/// Logs a message at the DEBUG level.
pub fn debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// Logs a message at the INFO level.
pub fn info(message: &str) {
    log(LogLevel::Info, message);
}

/// Logs a message at the WARN level.
pub fn warn(message: &str) {
    log(LogLevel::Warn, message);
}

/// Logs a message at the ERROR level.
pub fn error(message: &str) {
    log(LogLevel::Error, message);
}
