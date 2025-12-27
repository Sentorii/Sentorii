//! Contains the `ValueProvider` trait for parsing sources into `toml::Value` objects.

use crate::ConfigError;
use std::fs::read_to_string;
use std::path::PathBuf;
use toml::{Table, Value};

#[cfg(test)]
use std::collections::HashMap;

/// A trait that abstracts the loading and parsing of a configuration source into a `toml::Value`.
pub trait ValueProvider {
    /// Loads a `Value` from a given optional path.
    /// # Errors
    /// `ConfigError::TomlParseError` could arise due to invalid file format.
    fn load_from(&self, path_opt: Option<PathBuf>) -> Result<Option<Value>, ConfigError>;
}

/// The production `ValueProvider` that reads and parses TOML files from the filesystem.
pub struct SystemValueProvider;

impl ValueProvider for SystemValueProvider {
    fn load_from(&self, path_opt: Option<PathBuf>) -> Result<Option<Value>, ConfigError> {
        if let Some(path) = path_opt.filter(|p| p.is_file()) {
            let content = read_to_string(&path)?;
            if content.trim().is_empty() {
                return Ok(Some(Value::Table(Table::new())));
            }
            let table: Table = content
                .parse()
                .map_err(|e| ConfigError::TomlParseError { path, source: e })?;
            Ok(Some(Value::Table(table)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub enum MockError {
    TomlParse,
}

/// A mock `ValueProvider` for use in unit tests.
/// It can be pre-loaded with `Value`s or errors for specific paths.
#[cfg(test)]
pub struct MockValueProvider {
    values: HashMap<PathBuf, Result<Option<Value>, MockError>>,
}

#[cfg(test)]
impl MockValueProvider {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn add_value(&mut self, path: PathBuf, value: Value) {
        self.values.insert(path, Ok(Some(value)));
    }

    pub(crate) fn add_error(&mut self, path: PathBuf, error: MockError) {
        self.values.insert(path, Err(error));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
impl ValueProvider for MockValueProvider {
    fn load_from(&self, path_opt: Option<PathBuf>) -> Result<Option<Value>, ConfigError> {
        path_opt.map_or(Ok(None), |path| match self.values.get(&path).cloned() {
            Some(Ok(Some(value))) => Ok(Some(value)),
            Some(Ok(None)) | None => Ok(None),
            Some(Err(mock_error)) => {
                let real_error = match mock_error {
                    MockError::TomlParse => ConfigError::TomlParseError {
                        path: path.clone(),
                        source: "invalid key".parse::<Value>().unwrap_err(),
                    },
                };
                Err(real_error)
            }
        })
    }
}
