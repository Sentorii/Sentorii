use thiserror::Error;

/// An error that can occur while translating a high-level `CommandStep`
/// into a low-level `ExecutableCommand`.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CommandBuildError {
    #[error("Failed to resolve placeholder '{{ {0} }}': key not found in context")]
    MissingContextKey(String),
    #[error("Invalid placeholder syntax in template: {0}")]
    InvalidPlaceholderSyntax(String),
}
