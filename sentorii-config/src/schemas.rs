//! Defines the canonical configuration schema for Sentorii.

use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct TomlConfig {
    #[serde(default)]
    pub gitflow: Option<TomlGitflowConfig>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct TomlGitflowConfig {
    #[serde(default)]
    pub main: Option<String>,
    #[serde(default)]
    pub develop: Option<String>,
    #[serde(default)]
    pub prefixes: Option<TomlPrefixesConfig>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct TomlPrefixesConfig {
    #[serde(default)]
    pub feature: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub hotfix: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub gitflow: GitflowConfig,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GitflowConfig {
    pub main: String,
    pub develop: String,
    pub prefixes: PrefixesConfig,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PrefixesConfig {
    pub feature: String,
    pub release: String,
    pub hotfix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gitflow: GitflowConfig {
                main: "main".to_string(),
                develop: "develop".to_string(),
                prefixes: PrefixesConfig {
                    feature: "feature/".to_string(),
                    release: "release/".to_string(),
                    hotfix: "hotfix".to_string(),
                },
            },
        }
    }
}

impl Config {
    pub fn overlay(&mut self, loaded: TomlConfig) {
        if let Some(loaded_gitflow) = loaded.gitflow {
            self.gitflow.overlay(loaded_gitflow);
        }
    }
}

impl GitflowConfig {
    pub(crate) fn overlay(&mut self, loaded: TomlGitflowConfig) {
        if let Some(main_branch) = loaded.main {
            self.main = main_branch;
        }
        if let Some(develop_branch) = loaded.develop {
            self.develop = develop_branch;
        }
        if let Some(loaded_prefixes) = loaded.prefixes {
            self.prefixes.overlay(loaded_prefixes);
        }
    }
}

impl PrefixesConfig {
    pub(crate) fn overlay(&mut self, loaded: TomlPrefixesConfig) {
        if let Some(feature) = loaded.feature {
            self.feature = feature;
        }
        if let Some(release) = loaded.release {
            self.release = release;
        }
        if let Some(hotfix) = loaded.hotfix {
            self.hotfix = hotfix;
        }
    }
}
