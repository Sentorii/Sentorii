//! Defines the Rust structs that map directly to the `sentorii.toml` file format.

use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

/// The final, merged configuration struct that the rest of the application will use.
///
/// It is built by merging configurations from defaults, a global file, a project file,
/// and environment variables.
#[derive(Default, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Plugin-related configurations.
    pub plugins: Option<Plugins>,
}

/// A struct for the `[plugins]` section of the configuration.
#[derive(Default, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Plugins {
    /// Defines the plugin versioning strategy.
    pub versioning: Option<VersioningConfig>,
}

/// Defines the flexible plugin versioning configuration.
///
/// This enum can be specified in one of three ways in the TOML file:
/// 1. As the string `"auto"` for automatic behavior.
/// 2. As a single string specifying a plugin name.
/// 3. As an array of tables, each with a `path` and `plugin` key.
#[derive(Debug, PartialEq, Eq)]
pub enum VersioningConfig {
    /// Represents the "auto" versioning strategy.
    Auto,
    /// Specifies a single, default plugin by its name.
    Single(String),
    /// Specifies multiple plugins for different project paths.
    Multi(Vec<MultiPluginConfig>),
}

/// A struct for defining a plugin for a specific path in a multi-project setup.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MultiPluginConfig {
    /// The path to a specific project or directory.
    pub path: PathBuf,
    /// The plugin to be used for the specified path.
    pub plugin: String,
}

// Private helper for deserializing the VersioningConfig enum.
// This allows us to distinguish between the special "auto" string and other strings.
#[derive(Deserialize)]
#[serde(untagged)]
enum VersioningConfigHelper {
    Multi(Vec<MultiPluginConfig>),
    Str(String),
}

impl<'de> Deserialize<'de> for VersioningConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match VersioningConfigHelper::deserialize(deserializer)? {
            VersioningConfigHelper::Multi(m) => Ok(Self::Multi(m)),
            VersioningConfigHelper::Str(s) if s.to_lowercase() == "auto" => Ok(Self::Auto),
            VersioningConfigHelper::Str(s) => Ok(Self::Single(s)),
        }
    }
}
