//! Contains the logic for finding, loading, and merging configuration files.

use crate::error::ConfigError;
use crate::schemas::{Config, TomlConfig};
use serde_json::Value::Object;
use serde_json::{Map, Value};
use std::env;
use std::fs::read_to_string;
use std::io::ErrorKind;
use std::path::PathBuf;

/// Loads, merges, and validates configuration from all sources.
/// # Errors
///
/// This function will return an error if:
/// - A configuration file is found but cannot be read due to I/O issues (e.g., permissions).
/// - A configuration file contains syntactically invalid TOML.
pub fn load_config() -> Result<Config, ConfigError> {
    let global_content = find_global_config_path()
        .map(read_to_string)
        .transpose()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                ConfigError::Io(std::io::Error::other(e))
            } else {
                e.into()
            }
        });

    let global_content = match global_content {
        Ok(content) => content,
        Err(e) => {
            if e.to_string().contains("ignore") {
                None
            } else {
                return Err(e);
            }
        }
    };

    let project_content = find_git_root()
        .ok()
        .flatten()
        .map(|root| {
            let path = root.join(".sentorii").join("config.toml");
            read_to_string(path)
        })
        .transpose()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                ConfigError::Io(std::io::Error::other(e))
            } else {
                e.into()
            }
        });

    let project_content = match project_content {
        Ok(content) => content,
        Err(e) => {
            if e.to_string().contains("ignore") {
                None
            } else {
                return Err(e);
            }
        }
    };

    build_config(global_content, project_content, env::vars())
}

fn build_config(
    global_toml: Option<String>,
    project_toml: Option<String>,
    env_vars: impl IntoIterator<Item = (String, String)>,
) -> Result<Config, ConfigError> {
    let mut config = Config::default();

    if let Some(toml_str) = global_toml {
        let loaded: TomlConfig = toml::from_str(&toml_str)?;
        config.overlay(loaded);
    }

    if let Some(toml_str) = project_toml {
        let loaded: TomlConfig = toml::from_str(&toml_str)?;
        config.overlay(loaded);
    }

    if let Some(loaded) = parse_toml_from_key_values(env_vars) {
        config.overlay(loaded);
    }

    Ok(config)
}

fn parse_toml_from_key_values(
    env_vars: impl IntoIterator<Item = (String, String)>,
) -> Option<TomlConfig> {
    const PREFIX: &str = "SENTORII_";
    let mut root_map = Map::new();
    for (key, value) in env_vars.into_iter().filter(|(k, _)| k.starts_with(PREFIX)) {
        let path = key.trim_start_matches(PREFIX).to_lowercase();
        insert_value_by_path(&mut root_map, &path, Value::String(value));
    }
    if root_map.is_empty() {
        None
    } else {
        serde_json::from_value(Object(root_map)).ok()
    }
}

fn insert_value_by_path(root_map: &mut Map<String, Value>, path: &str, value: Value) {
    let parts: Vec<_> = path.split('_').collect();

    if let Some((final_key, path_segments)) = parts.split_last() {
        let mut current_map = root_map;

        for segment in path_segments {
            let entry = current_map
                .entry((*segment).to_string())
                .or_insert_with(|| Object(Map::new()));

            if let Object(next_map) = entry {
                current_map = next_map;
            } else {
                return;
            }
        }
        current_map.insert((*final_key).to_string(), value);
    }
}

fn find_global_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "sentorii", "Sentorii")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

fn find_git_root() -> std::io::Result<Option<PathBuf>> {
    let start_dir = env::current_dir()?;
    let mut current_dir = start_dir.as_path();

    loop {
        if current_dir.join(".git").is_dir() {
            return Ok(Some(current_dir.to_path_buf()));
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => return Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_environment_variable_parsing() {
        let mock_env_vars = vec![
            ("IRRELEVANT_VAR".to_string(), "foo".to_string()),
            ("SENTORII_GITFLOW_MAIN".to_string(), "env_main".to_string()),
            (
                "SENTORII_GITFLOW_PREFIXES_FEATURE".to_string(),
                "feat/".to_string(),
            ),
        ];

        let config_from_env_vars = parse_toml_from_key_values(mock_env_vars);

        let loaded_config = config_from_env_vars
            .map_or_else(|| panic!("config_from_env_vars is empty"), |config| config);
        let gitflow = loaded_config.gitflow.map_or_else(
            || panic!("parse_toml_from_key_values failed"),
            |gitflow| gitflow,
        );
        let prefixes = gitflow.prefixes.map_or_else(
            || panic!("parse_toml_from_key_values failed"),
            |prefixes| prefixes,
        );

        assert_eq!(gitflow.main, Some("env_main".to_string()));
        assert_eq!(prefixes.feature, Some("feat/".to_string()));
        assert!(gitflow.develop.is_none());
    }
}
