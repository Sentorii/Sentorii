//! Provides a testable abstraction and parsing logic for the process environment.

use crate::ConfigError;
use toml::{Table, Value};

#[cfg(test)]
use std::collections::HashMap;

const ENV_PREFIX: &str = "sentorii_";
const ENV_SEPARATOR: &str = "_";

/// A trait that abstracts the source of environment variables.
pub trait EnvironmentProvider {
    /// Returns an iterator over all environment variables as (key, value) pairs.
    fn vars(&self) -> Box<dyn Iterator<Item = (String, String)>>;
}

/// The production `EnvironmentProvider` that reads from the actual process.
pub struct ProcessEnvironmentProvider;

impl EnvironmentProvider for ProcessEnvironmentProvider {
    fn vars(&self) -> Box<dyn Iterator<Item = (String, String)>> {
        Box::new(std::env::vars())
    }
}

/// A mock `EnvironmentProvider` for use in tests.
#[cfg(test)]
pub(crate) struct MockEnvironmentProvider {
    vars: HashMap<String, String>,
}

#[cfg(test)]
impl MockEnvironmentProvider {
    /// Creates a new, empty mock environment.
    pub(crate) fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, key: &str, raw_value: &str) {
        self.vars.insert(key.to_string(), raw_value.to_string());
    }
}

#[cfg(test)]
impl EnvironmentProvider for MockEnvironmentProvider {
    fn vars(&self) -> Box<dyn Iterator<Item = (String, String)>> {
        Box::new(self.vars.clone().into_iter())
    }
}

/// Builds a nested `toml::Value` from a set of environment variables.
///
/// This is the core of the introspective environment variable laoding. It iterates
/// through variables provided by an `EnvironmentProvider`, filters for the ones
/// starting with `SENTORII_`, and constructs a `toml::Value` that mirrors
/// the `Config` struct's shape.
pub(crate) fn build_value_from_env<E: EnvironmentProvider>(env: &E) -> Result<Value, ConfigError> {
    let mut root_table = Table::new();

    for (key, value_str) in env.vars() {
        let lower_key = key.to_lowercase();

        if !lower_key.starts_with(ENV_PREFIX) {
            continue;
        }

        let path = lower_key
            .trim_start_matches(ENV_PREFIX)
            .split(ENV_SEPARATOR)
            .collect::<Vec<_>>();

        let value = match value_str.parse::<Value>() {
            Ok(parsed_value) => parsed_value,
            Err(e) => {
                let message = e.message();
                if message.contains("expected literal string")
                    || message.contains("expected a value")
                        && !value_str.starts_with('[')
                        && !value_str.ends_with(']')
                {
                    Value::String(value_str.clone())
                } else {
                    return Err(ConfigError::EnvVarTomlParse {
                        var_name: key,
                        source: e,
                    });
                }
            }
        };

        insert_value_by_path(&mut root_table, &path, value);
    }

    Ok(Value::Table(root_table))
}

/// Recursively inserts a value into a `toml::Table` following a path of keys.
fn insert_value_by_path(current_table: &mut Table, path: &[&str], value: Value) {
    let segment = path[0];

    if path.len() == 1 {
        current_table.insert(segment.to_string(), value);
        return;
    }

    let next_table = current_table
        .entry(segment.to_string())
        .or_insert_with(|| Value::Table(Table::new()));

    if let Value::Table(table) = next_table {
        insert_value_by_path(table, &path[1..], value);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use toml::Value::Table;
    use toml::toml;

    #[test]
    fn test_build_value_from_single_simple_var() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_MAIN", "value");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { main = "value" }));
    }

    #[test]
    fn test_build_value_from_nested_var() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_TABLE_KEY", "value");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { table = { key = "value" }}));
    }

    #[test]
    fn test_build_value_is_case_insensitive() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("sentorii_lower", "1");
        mock_env.add("SENTORII_UPPER", "2");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { lower = 1 upper = 2 }));
    }

    #[test]
    fn test_build_value_from_multiple_vars_merges_correctly() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_A", "1");
        mock_env.add("SENTORII_B", "true");
        mock_env.add("SENTORII_TABLE_A", "val");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { a = 1 b = true table = { a = "val" }}));
    }

    #[test]
    fn test_build_value_parses_toml_string() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_KEY", "my-value");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { key = "my-value" }));
    }

    #[test]
    fn test_build_value_parses_toml_quoted_string() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_KEY", "\"my-value\"");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { key = "my-value" }));
    }

    #[test]
    fn test_build_value_parses_toml_integer() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_KEY", "123");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { key = 123 }));
    }

    #[test]
    fn test_build_value_parses_toml_boolean() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_KEY", "true");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { key = true }));
    }

    #[test]
    fn test_build_value_parses_toml_array() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_KEY", "[1, \"two\"]");
        let value = build_value_from_env(&mock_env).unwrap();
        let expected = Table(toml::from_str("key = [1, \"two\"]").unwrap());
        assert_eq!(value, expected);
    }

    #[test]
    fn test_build_value_ignores_vars_without_prefix() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("OTHER_VAR", "123");
        mock_env.add("SENTORII_A", "1");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { a = 1}));
    }

    #[test]
    fn test_build_value_returns_error_for_malformed_toml_value() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_A", "[1,2,");
        let result = build_value_from_env(&mock_env);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::EnvVarTomlParse { var_name, .. } => {
                assert_eq!(var_name, "SENTORII_A");
            }
            _ => panic!("Expected EnvVarTomlParse error"),
        }
    }

    #[test]
    fn test_build_value_handles_string_that_looks_like_malformed_number() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_A", "123a");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { a = "123a" }));
    }

    #[test]
    fn test_build_value_handles_conflicting_paths_gracefully() {
        let mut mock_env = MockEnvironmentProvider::new();
        mock_env.add("SENTORII_A_B", "1");
        mock_env.add("SENTORII_A_B_C", "2");
        let value = build_value_from_env(&mock_env).unwrap();
        assert_eq!(value, Table(toml! { a = { b = 1 }}));
    }
}
