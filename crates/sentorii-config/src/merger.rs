//! Contains the `Merge` trait and its implementation for `toml::Value`.

use toml::Value;

/// A trait for deep-merging one object into another.
pub trait Merge {
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

#[cfg(test)]
mod tests {
    use super::*;
    use Value::Table;
    use toml::toml;

    #[test]
    fn test_merge_simple_tables() {
        let mut target = Table(toml! { a = 1 });
        let source = Table(toml! { b = 2 });
        target.merge(source);
        assert_eq!(
            target,
            Table(toml! {
                a = 1 b = 2
            })
        );
    }

    #[test]
    fn test_merge_overwrites_existing_key() {
        let mut target = Table(toml! { a = 1 b = 1 });
        let source = Table(toml! { b = 2 c = 3});
        target.merge(source);
        assert_eq!(target, Table(toml! { a = 1 b = 2 c = 3}));
    }

    #[test]
    fn test_merge_deeply_nested_tables() {
        let mut target = Table(toml! {
            a = { b = { c = 1 }}
        });
        let source = Table(toml! {
            a = { b = { d = 2 }, e = 3}
        });
        target.merge(source);
        assert_eq!(
            target,
            Table(toml! {
                a = { b = { c = 1, d = 2 }, e = 3 }
            })
        );
    }

    #[test]
    fn test_merge_non_table_value_replaces() {
        let mut target = Table(toml! { a = "original" });
        let source = Table(toml! { a = "replaced" });
        target.merge(source);
        assert_eq!(target, Table(toml! { a = "replaced" }));
    }

    #[test]
    fn test_merge_value_overwrites_table() {
        let mut target = Table(toml! { a = { b = 1 }});
        let source = Table(toml! { a = "replaced" });
        target.merge(source);
        assert_eq!(target, Table(toml! { a = "replaced" }));
    }

    #[test]
    fn test_merge_table_overwrites_value() {
        let mut target = Table(toml! { a = "original" });
        let source = Table(toml! { a = { b = 1} });
        target.merge(source);
        assert_eq!(target, Table(toml! { a = { b = 1 } }));
    }

    #[test]
    fn test_merge_empty_source_table_does_nothing() {
        let mut target = Table(toml! { a = 1 });
        let source = Table(toml::Table::new());
        target.merge(source);
        assert_eq!(target, Table(toml! { a = 1 }));
    }

    #[test]
    fn test_merge_into_empty_target_table_populates_it() {
        let mut target = Table(toml::Table::new());
        let source = Table(toml! { a = 1 });
        target.merge(source);
        assert_eq!(target, Table(toml! { a = 1 }));
    }
}
