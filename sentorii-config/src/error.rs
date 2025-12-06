//! Defines the crate's custom error type `ConfigError`.

use thiserror::Error;

/// The primary error type for the `sentorii-config` crate.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Wraps errors originating from the `figment` configuration library.
    ///
    /// This can include file I/O errors, TOML parsing errors (including those
    /// from `deny_unknown_fields`), and more.
    #[error("Configuration loading failed: {0}")]
    LoadError(#[from] figment::Error),

    /// Occurs when the user's home or config directory cannot be determined.
    ///
    /// This is necessary for locating the global configuration file.
    #[error("Could not determine user's home or config directory")]
    HomeDirNotFound,
}