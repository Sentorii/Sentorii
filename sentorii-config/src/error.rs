//! Defines the crate's custom error type, `ConfigError`.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    /// An error occurred while serializing the default configuration struct into a TOML value.
    /// This is an internal error and should rarely, if ever, occur.
    #[error("Failed to serialize default config: {0}")]
    Serialization(#[from] toml::ser::Error),

    /// An error occurred while deserializing the final merged TOML Value into the `Config` struct.
    /// This typically happens if an environment variable sets a value of the wrong type
    /// (e.g., `SENTORII_BRANCHING="a-string"`) or contains an unknown field.
    #[error("Failed to build final configuration: {0}")]
    Deserialization(#[from] toml::de::Error),

    /// An error occurred while parsing a TOML configuration file.
    /// The error includes the path to the malformed file.
    #[error("Failed to parse TOML file at '{path}': {source}")]
    TomlParseError {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// An error occurred while parsing the value of an environment variable as TOML.
    /// This can happen if a complex value (like a list of tables) is malformed.
    #[error("Failed to parse TOML value for environment variable '{var_name}': {source}")]
    EnvVarTomlParse {
        var_name: String,
        source: toml::de::Error,
    },

    /// An error occurred during file I/O, such as a permissions issue or the `git`
    /// command failing to spawn.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The user's home directory could not be determined, preventing the global
    /// configuration file from being found.
    #[error("Could not determine user's home directory")]
    HomeDirNotFound,
}
