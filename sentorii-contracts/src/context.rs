use crate::error::CommandBuildError;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines the `Context` struct and automatically implements safe, strongly-typed
/// getter methods for each of its core fields.
///
/// This macro is the single source of truth for the workflow context schema.
/// It avoids repetitive boilerplate and ensures that adding a new core field is
/// a simple, single-line change.
///
/// ### How to Add a New Core Field
///
/// 1.  Add a new line to the macro invocation below (e.g., `/// The name of the release branch. pub release_branch: Option<String>`).
/// 2.  That's it. The macro will automatically:
///     - Add the field to the `Context` struct.
///     - Generate a corresponding getter method (e.g., `pub fn get_release_branch(&self) -> Result<&str, ...>`).
macro_rules! context {
    (
        $(
            $(#[$field_meta:meta])*
            $field_name:ident: Option<$field_type:ty>
        ),*
    ) => {
        /// The single, strongly-typed container for all data available during a workflow.
        #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
        pub struct Context {
            // --- Auto-generated "Simple" Fields ---
            $(
                $(#[$field_meta])*
                $field_name: Option<$field_type>,
            )*

            /// A flexible bucket for custom data.
            #[serde(default)]
            custom: HashMap<String, String>,
        }

        impl Context {
            paste! {
                $(
                    $(#[$field_meta])*
                    pub fn [<get_ $field_name>](&self) -> Result<&$field_type, CommandBuildError> {
                        self.$field_name.as_ref().ok_or(CommandBuildError::MissingContextKey(
                            stringify!($field_name).to_string(),
                        ))
                    }

                    $(#[$field_meta])*
                    pub fn [<set_ $field_name>](&mut self, value: impl Into<$field_type>) -> &mut Self {
                        self.$field_name = Some(value.into());
                        self
                    }
                )*
            }
        }

        #[derive(Debug, Default)]
        pub struct ContextBuilder {
            context: Context,
        }

        impl ContextBuilder {
            pub fn new() -> Self { Self::default() }
            pub fn build(self) -> Context { self.context }

            paste! {
                $(
                    pub fn [<with_ $field_name>](mut self, value: impl Into<$field_type>) -> Self {
                        self.context.[<set_ $field_name>](value);
                        self
                    }
                )*
            }
        }
    };
}

context!(
    main: Option<String>,
    develop: Option<String>,
    remote: Option<String>
);
