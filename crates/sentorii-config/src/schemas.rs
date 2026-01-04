//! Defines the canonical configuration schema for Sentorii.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub plugins: Plugins,
    #[serde(default)]
    pub branching: Branching,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Plugins {
    pub versioning: Option<VersioningConfig>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct Branching {
    pub main: String,
    pub develop: String,
    pub remote: String,
    pub prefixes: Prefixes,
}

impl Default for Branching {
    fn default() -> Self {
        Self {
            main: "main".to_string(),
            develop: "develop".to_string(),
            remote: "origin".to_string(),
            prefixes: Prefixes::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct Prefixes {
    pub feature: String,
    pub release: String,
    pub hotfix: String,
}

impl Default for Prefixes {
    fn default() -> Self {
        Self {
            feature: "feature/".to_string(),
            release: "release/".to_string(),
            hotfix: "hotfix/".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum VersioningConfig {
    Auto,
    Single(String),
    Multi(Vec<MultiPluginConfig>),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct MultiPluginConfig {
    pub path: PathBuf,
    pub plugin: String,
}
