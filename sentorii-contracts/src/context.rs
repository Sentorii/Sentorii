use crate::error::CommandBuildError;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueSource {
    Literal(String),
    FromContext(ContextKey),
}

impl From<&str> for ValueSource {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_string())
    }
}
impl From<String> for ValueSource {
    fn from(s: String) -> Self {
        Self::Literal(s)
    }
}
impl From<ContextKey> for ValueSource {
    fn from(s: ContextKey) -> Self {
        Self::FromContext(s)
    }
}

/// Defines the `Context` struct, the `ContextKey` enum, the `ContextBuilder`,
/// and all boilerplate getters and setters.
///
/// This macro is the single source of truth for the context *schema*. It eliminates
/// repetitive code, ensuring that adding a new core field is a simple, safe,
/// single-line change to the invocation below.
///
/// Business logic, such as mapping from a `Config` struct, is implemented in a
/// separate, handwritten `impl` block for the `ContextBuilder`.
macro_rules! context {
    (
        $(#[$struct_meta:meta])*
        $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        // --- Part A: The `ContextKey` Enum ---
        paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            pub enum ContextKey {
                $(
                    #[doc = "Represents the `"]
                    #[doc = stringify!($field_name)]
                    #[doc = "` field in the Context."]
                    [< $field_name:camel >],
                )*
            }
        }

        // --- Part B: The `Context` Struct ---
        $(#[$struct_meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Context {
            $(
                $(#[$field_meta])*
                $field_name: Option<$field_type>,
            )*
        }

        // --- Part C: The `Context` `impl` block (Getters and Setters) ---
        impl Context {
            paste! {
                $(
                    /// # Errors
                    /// Can fail on missing `ContextKey`.
                    pub fn [<get_ $field_name>](&self) -> Result<&$field_type, CommandBuildError> {
                        self.$field_name.as_ref().ok_or(CommandBuildError::MissingContextKey(ContextKey::[< $field_name:camel >]))
                    }
                    pub fn [<set_ $field_name>](&mut self, value: impl Into<$field_type>) -> &mut Self {
                        self.$field_name = Some(value.into());
                        self
                    }
                )*
            }
        }

        // --- Part D: The `ContextBuilder` ---
        #[derive(fmt::Debug, Default)]
        pub struct ContextBuilder { context: Context }
        impl ContextBuilder {
            pub fn new() -> Self { Self::default() }
            pub fn build(self) -> Context { self.context }
            paste! {
                $(
                    #[must_use]
                    pub fn [<with_ $field_name>](mut self, value: impl Into<$field_type>) -> Self {
                        self.context.[<set_ $field_name>](value);
                        self
                    }
                )*
            }
        }

        // --- Part E: The `ValueSource::resolve` Logic ---
        impl ValueSource {
            /// Resolves the `ValueSource` to a concrete `String` using the provided context.
            /// # Errors
            /// Can fail to resolve placeholders from context with `CommandBuildError`.
            pub fn resolve(&self, context: &Context) -> Result<String, CommandBuildError> {
                match self {
                    ValueSource::Literal(s) => Ok(s.clone()),
                    ValueSource::FromContext(key) => {
                        paste! {
                            match key {
                                $(
                                    ContextKey::[< $field_name:camel >] => {
                                        context.[<get_ $field_name>]().map(ToString::to_string)
                                    }
                                ),*
                            }
                        }
                    }
                }
            }
        }
    };
}

context!(Context {
    main: String,
    develop: String,
    remote: String,
    feature_branch: String,
    prefix_feature: String,
    tag: String
});

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_value_source_from_str() {
        let str = "hello";
        let value = ValueSource::from(str);
        assert_eq!(value, ValueSource::Literal(str.into()));
    }

    #[test]
    fn test_value_source_from_string() {
        let str = "hello".to_string();
        let value = ValueSource::from(str.clone());
        assert_eq!(value, ValueSource::Literal(str.into()));
    }

    #[test]
    fn test_value_source_from_context() {
        let key = ContextKey::Develop;
        let value = ValueSource::from(key.clone());
        assert_eq!(value, ValueSource::FromContext(key));
    }

    #[test]
    fn test_get_context() {
        let value = "test";
        let context = ContextBuilder::new().with_develop(value).build();
        assert_eq!(value, context.get_develop().unwrap());
    }

    #[test]
    fn test_set_context() {
        let value = "test";
        let mut context = ContextBuilder::new().build();
        context.set_develop(value);
        assert_eq!(value, context.get_develop().unwrap());
    }
}
