use sentorii_api::logging::{LogEntry, LogLevel, LogSource};
use std::io;
use std::io::Write;

/// Writes a log entry to stderr with the given level and message.
///
/// This function locks stderr, serializes the log entry to JSON, and writes it,
/// followed by a newline. It is the core logging primitive used by the public
/// logging functions.
pub fn log(level: LogLevel, message: String, source: LogSource) {
    let entry = LogEntry {
        level,
        message,
        source,
    };
    if let Ok(json) = serde_json::to_string(&entry) {
        let mut stderr = io::stderr().lock();
        let _ = writeln!(stderr, "{json}");
    }
}

/// Logs a message at the DEBUG level.
pub fn debug(message: &str) {
    log(LogLevel::Debug, message.to_string(), LogSource::Pdk);
}

/// Logs a message at the INFO level.
pub fn info(message: &str) {
    log(LogLevel::Info, message.to_string(), LogSource::Pdk);
}

/// Logs a message at the WARN level.
pub fn warn(message: &str) {
    log(LogLevel::Warn, message.to_string(), LogSource::Pdk);
}

/// Logs a message at the ERROR level.
pub fn error(message: &str) {
    log(LogLevel::Error, message.to_string(), LogSource::Pdk);
}
