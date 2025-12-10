//! Contains the orchestration logic for loading configuration from all sources.

use crate::env::{EnvironmentProvider, ProcessEnvironmentProvider, build_value_from_env};
use crate::merger::Merge;
use crate::parser::{SystemValueProvider, ValueProvider};
use crate::paths::{
    GlobalPathProvider, ProjectPathProvider, SystemGlobalPathProvider, SystemProjectPathProvider,
};
use crate::{Config, ConfigError};
use toml::Value;

/// The primary, zero-maintenance configuration loader.
///
/// This function is the "composition root" where we instantiate the
/// concrete providers for the real environment and filesystem. It orchestrates
/// the entire configuration loading and merging process, which proceeds in the
/// following order of precedence (where later sources override earlier ones):
///
/// 1.  **Built-in Defaults:** The `Config::default()` values.
/// 2.  **Global File:** A global `config.toml` (e.g., at `~/.config/sentorii/config.toml`).
/// 3.  **Project File:** A project-specific `.sentorii/config.toml` at the Git repository root.
/// 4.  **Environment Variables:** Variables prefixed with `SENTORII_`.
///
/// # Errors
///
/// This function will return an error under the following conditions, bubbling
/// up the most relevant issue encountered during the loading process:
///
/// *   Returns [`ConfigError::IoError`] if there is a problem reading a configuration
///     file (e.g., due to file permissions) or if the `git` command fails to
///     execute (e.g., it is not in the system's `PATH`).
///
/// *   Returns [`ConfigError::TomlParseError`] if a configuration file
///     (`config.toml`) contains invalid TOML syntax. The
///     error will include the path to the malformed file.
///
/// *   Returns [`ConfigError::EnvVarTomlParse`] if an environment variable's
///     value is not a valid TOML fragment (e.g., a complex array string is
///     malformed).
///
/// *   Returns [`ConfigError::Deserialization`] if the final, merged configuration
///     does not match the shape of the `Config` struct. This is most commonly
///     caused by a type mismatch (e.g., `SENTORII_BRANCHING="a-string"`) or a
///     typo in a key name in any configuration source (due to `#[serde(deny_unknown_fields)]`).
///
/// *   Returns [`ConfigError::Serialization`] in the unlikely event that the
///     default `Config` struct cannot be serialized into a TOML value. This
///     is an internal error and should not typically occur.
///
pub fn load_config() -> Result<Config, ConfigError> {
    let global_paths = SystemGlobalPathProvider::new(dirs::home_dir());
    let project_paths = SystemProjectPathProvider;
    let env = ProcessEnvironmentProvider;
    let value_provider = SystemValueProvider;
    load_config_from(&global_paths, &project_paths, &env, &value_provider)
}

/// Internal loader function generic over all provider traits.
pub(crate) fn load_config_from<G, P, E, V>(
    global_paths: &G,
    project_paths: &P,
    env: &E,
    value_provider: &V,
) -> Result<Config, ConfigError>
where
    G: GlobalPathProvider,
    P: ProjectPathProvider,
    E: EnvironmentProvider,
    V: ValueProvider,
{
    let mut merged_config = Value::try_from(Config::default())?;

    let global_path = global_paths.global_config_path();
    if let Some(global_value) = value_provider.load_from(global_path)? {
        merged_config.merge(global_value);
    }

    let project_path = project_paths.project_config_path()?;
    if let Some(project_value) = value_provider.load_from(project_path)? {
        merged_config.merge(project_value);
    }

    let env_value = build_value_from_env(env)?;
    merged_config.merge(env_value);

    Ok(merged_config.try_into()?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::env::MockEnvironmentProvider;
    use crate::parser::{MockError, MockValueProvider};
    use crate::paths::{MockGlobalPathProvider, MockProjectPathProvider};
    use crate::schemas::{Branching, Plugins, Prefixes};
    use std::path::PathBuf;
    use toml::toml;

    #[test]
    fn logic_test_full_hierarchy_merge() {
        let global_path = PathBuf::from("/fake/home/.config/sentorii/sentorii.toml");
        let project_path = PathBuf::from("/fake/repo/sentorii.toml");

        let global_value = toml! {
            [branching]
            main = "global-main"
        };
        let project_value = toml! {
            [branching]
            develop = "project-dev"
        };

        let mock_global_paths = MockGlobalPathProvider {
            path: Some(global_path.clone()),
        };
        let mock_project_paths = MockProjectPathProvider {
            path: Ok(Some(project_path.clone())),
        };

        let mut mock_value_provider = MockValueProvider::new();
        mock_value_provider.add_value(global_path, Value::from(global_value));
        mock_value_provider.add_value(project_path, Value::from(project_value));

        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add_string("SENTORII_BRANCHING_MAIN", "env-main");
        mock_env.add_string("SENTORII_BRANCHING_PREFIXES_HOTFIX", "urgent/");

        let config = load_config_from(
            &mock_global_paths,
            &mock_project_paths,
            &mock_env,
            &mock_value_provider,
        )
        .unwrap();

        let expected = Config {
            plugins: Plugins::default(),
            branching: Branching {
                main: "env-main".to_string(),
                develop: "project-dev".to_string(),
                prefixes: Prefixes {
                    feature: "feature/".to_string(),
                    release: "release/".to_string(),
                    hotfix: "urgent/".to_string(),
                },
            },
        };
        assert_eq!(config, expected);
    }

    #[test]
    fn logic_test_error_on_type_mismatch() {
        let mock_global_paths = MockGlobalPathProvider { path: None };
        let mock_project_paths = MockProjectPathProvider { path: Ok(None) };
        let mut mock_env = MockEnvironmentProvider::new();
        let mock_value_provider = MockValueProvider::new();
        mock_env.add_string("SENTORII_BRANCHING", "a-string");

        let result = load_config_from(
            &mock_global_paths,
            &mock_project_paths,
            &mock_env,
            &mock_value_provider,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::Deserialization(e) => {
                assert!(e.to_string().contains("invalid type: string"));
            }
            other => panic!("Expected Deserialization error, got {other:?}"),
        }
    }

    #[test]
    fn unit_test_propagates_error_from_value_provider() {
        let global_path = PathBuf::from("/global.toml");
        let mock_global_paths = MockGlobalPathProvider {
            path: Some(global_path.clone()),
        };
        let mock_project_paths = MockProjectPathProvider { path: Ok(None) };
        let mock_env = MockEnvironmentProvider::new();
        let mut mock_value_provider = MockValueProvider::new();

        mock_value_provider.add_error(global_path.clone(), MockError::TomlParse);

        let result = load_config_from(
            &mock_global_paths,
            &mock_project_paths,
            &mock_env,
            &mock_value_provider,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::TomlParseError { path, .. } => {
                assert_eq!(path, global_path);
            }
            other_error => {
                panic!("Expected a TomlParseError, but got {other_error:?}");
            }
        }
    }
}
