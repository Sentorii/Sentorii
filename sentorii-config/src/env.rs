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

    pub(crate) fn add_string(&mut self, key: &str, raw_string_value: &str) {
        self.vars
            .insert(key.to_string(), format!("\"{raw_string_value}\""));
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

        let value = value_str
            .parse::<Value>()
            .map_err(|e| ConfigError::EnvVarTomlParse {
                var_name: lower_key.clone(),
                source: e,
            })?;

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
