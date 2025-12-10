//! Contains the `Merge` trait and its implementation for `toml::Value`.

use toml::Value;

/// A trait for deep-merging one object into another.
pub(crate) trait Merge {
    /// Deeply merges a `source` object into `self`.
    fn merge(&mut self, source: Self);
}

impl Merge for Value {
    fn merge(&mut self, source: Self) {
        if let Self::Table(target_table) = self
            && let Self::Table(source_table) = source
        {
            for (key, value) in source_table {
                if let Some(target_value) = target_table.get_mut(&key) {
                    target_value.merge(value);
                } else {
                    target_table.insert(key, value);
                }
            }
            return;
        }
        *self = source;
    }
}
