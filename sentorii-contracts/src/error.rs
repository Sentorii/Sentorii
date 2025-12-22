use crate::context::ContextKey;
use thiserror::Error;

/// An error that can occur while translating a high-level `CommandStep`
/// into a low-level `ExecutableCommand`.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CommandBuildError {
    #[error("Failed to resolve placeholder '{{ {0:?} }}': key not found in context")]
    MissingContextKey(ContextKey),
    #[error("Invalid placeholder syntax in template: {0}")]
    InvalidPlaceholderSyntax(String),
}

/// An error that occurred during the execution of a system command.
#[derive(Debug, Error)]
pub enum CommandExecutionError {
    #[error("Failed to spawn command '{command}': {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Command '{command}' failed to complete: {source}")]
    Wait {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Command '{command}' exited with non-zero status: {status}")]
    NonZeroStatus { command: String, status: String },
    #[error("A task essential for command execution panicked")]
    TaskPanic,
    /// Occurs when the underlying lock on internal state (e.g., for mocks) is poisoned.
    #[error("Mutex lock was poisoned")]
    LockPoisoned,
    /// A generic error that can be used for other execution failures.
    #[error("Command execution failed: {0}")]
    General(String),
}
