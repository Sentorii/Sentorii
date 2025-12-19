//! Defines different versions of steps.

use crate::command::Command;
use crate::command::ExecutableCommand;
use crate::command::{
    GitBranchDeleteCommand, GitCheckOutCommand, GitCheckoutNewBranchCommand, GitMergeNoFfCommand,
    GitPullCommand, GitPushCommand, GitPushTagsCommand, GitStatusCheckCommand, GitTagCommand,
    PluginExecuteCommand,
};
use crate::context::Context;
use crate::context::ValueSource;
use crate::error::CommandBuildError;
use crate::event::{RecoveryAction, RevertAction, StaticStepInfo};
use paste::paste;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A high-level, UI-friendly category for a command step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Check,
    Checkout,
    Pull,
    Merge,
    Push,
    Tag,
    DeleteBranch,
    Plugin,
    UserInteraction,
}

/// The declarative representation of a single step within a workflow plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    /// A step that executes a command.
    Command(CommandStep),
    /// A step that pauses the workflow to request a string input from the user.
    RequestStringInput(RequestStringInputTemplate),
    /// A step that pauses the workflow to request a selection from the user.
    RequestSelectInput(RequestSelectInputTemplate),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestStringInputTemplate {
    /// A unique key to identify this input request.
    key: String,
    /// The message to display to the user.
    prompt: String,
    /// An optional default value for the input.
    default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestSelectInputTemplate {
    /// A unique key to identify this input request.
    key: String,
    /// The message to display to the user.
    prompt: String,
    /// A list of options to display to the user.
    options: Vec<String>,
    /// An optional default option to pre-select.
    default_option: Option<String>,
}

impl Step {
    pub fn static_info(&self) -> StaticStepInfo {
        let (description, category) = match self {
            Step::Command(cmd) => (cmd.static_description(), cmd.category()),
            Step::RequestStringInput(req) => (req.prompt.clone(), Category::UserInteraction),
            Step::RequestSelectInput(req) => (req.prompt.clone(), Category::UserInteraction),
        };

        StaticStepInfo {
            description,
            category,
        }
    }
}

/// Defines the `CommandStep` enum and automatically implements the `Command` trait for it.
///
/// This macro is the single source of truth for all commands in the system.
/// To add a new command:
/// 1. Create a new struct (e.g., `MyNewCommand`).
/// 2. Implement the `Command` trait for `MyNewCommand`.
/// 3. Add one line to this macro invocation: `MyNew(MyNewCommand)`.
macro_rules! command_step {
    ( $(
            $(#[$variant_meta:meta])*
            $helper_name:ident ( $( $param_name:ident: $param_type:ty ),* ) => $variant:ident($command_type:ty)
        ),* ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum CommandStep {
            $( $(#[$variant_meta])* $variant($command_type), )*
        }

        impl Command for CommandStep {
            fn static_description(&self) -> String {
                match self { $( Self::$variant(cmd) => cmd.static_description(), )* }
            }
            fn resolved_description(&self, _context: &Context) -> String {
                match self { $( Self::$variant(cmd) => cmd.resolved_description(_context), )* }
            }
            fn category(&self) -> Category {
                match self { $( Self::$variant(cmd) => cmd.category(), )* }
            }
            fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
                match self { $( Self::$variant(cmd) => cmd.to_executable(context), )* }
            }
            fn possible_reverts(&self) -> Vec<RevertAction> {
                match self { $( Self::$variant(cmd) => cmd.possible_reverts(), )* }
            }
            fn possible_recoveries(&self) -> Vec<RecoveryAction> {
                match self { $( Self::$variant(cmd) => cmd.possible_recoveries(), )* }
            }
        }

        paste! {
            $(
                $(#[$variant_meta])*
                pub fn $helper_name( $( $param_name: $param_type ),* ) -> Step {
                    Step::Command(CommandStep::$variant($command_type {
                        $(
                            $param_name: $param_name.into(),
                        )*
                    }))
                }
            )*
        }
    };
}

command_step!(
    git_status_check() => GitStatusCheck(GitStatusCheckCommand),
    git_checkout(branch: impl Into<ValueSource>) => GitCheckout(GitCheckOutCommand),
    git_checkout_new_branch(branch: impl Into<ValueSource>) => GitCheckoutNewBranch(GitCheckoutNewBranchCommand),
    git_pull(remote: impl Into<ValueSource>, branch: impl Into<ValueSource>) => GitPull(GitPullCommand),
    git_merge(branch: impl Into<ValueSource>) => GitMergeNoFf(GitMergeNoFfCommand),
    git_push(remote: impl Into<ValueSource>, branch: impl Into<ValueSource>) => GitPush(GitPushCommand),
    git_tag(tag: impl Into<ValueSource>) => GitTag(GitTagCommand),
    git_push_tags() => GitPushTags(GitPushTagsCommand),
    git_delete_branch(branch: impl Into<ValueSource>) => GitDeleteBranch(GitBranchDeleteCommand),
    plugin_execute(executable: impl Into<String>, args: Vec<String>) => PluginExecute(PluginExecuteCommand)
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ContextKey, ValueSource};

    #[test]
    fn test_git_status_check_constructor() {
        let actual = git_status_check();
        let expected = Step::Command(CommandStep::GitStatusCheck(GitStatusCheckCommand));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_checkout_constructor() {
        let actual = git_checkout("test");
        let expected = Step::Command(CommandStep::GitCheckout(GitCheckOutCommand {
            branch: ValueSource::Literal("test".to_string()),
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_checkout_new_branch_constructor() {
        let actual = git_checkout_new_branch("test");
        let expected = Step::Command(CommandStep::GitCheckoutNewBranch(
            GitCheckoutNewBranchCommand {
                branch: ValueSource::Literal("test".to_string()),
            },
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_pull_constructor() {
        let actual = git_pull(ContextKey::Remote, "test");
        let expected = Step::Command(CommandStep::GitPull(GitPullCommand {
            remote: ValueSource::FromContext(ContextKey::Remote),
            branch: ValueSource::Literal("test".to_string()),
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_merge_constructor() {
        let actual = git_merge("test");
        let expected = Step::Command(CommandStep::GitMergeNoFf(GitMergeNoFfCommand {
            branch: ValueSource::Literal("test".to_string()),
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_constructor() {
        let actual = git_push(ContextKey::Remote, "test");
        let expected = Step::Command(CommandStep::GitPush(GitPushCommand {
            remote: ValueSource::FromContext(ContextKey::Remote),
            branch: ValueSource::Literal("test".to_string()),
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_tag_constructor() {
        let actual = git_tag("test");
        let expected = Step::Command(CommandStep::GitTag(GitTagCommand {
            tag: ValueSource::Literal("test".to_string()),
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_tags_constructor() {
        let actual = git_push_tags();
        let expected = Step::Command(CommandStep::GitPushTags(GitPushTagsCommand));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_delete_branch_constructor() {
        let actual = git_delete_branch("test");
        let expected = Step::Command(CommandStep::GitDeleteBranch(GitBranchDeleteCommand {
            branch: ValueSource::Literal("test".to_string()),
        }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_plugin_execute_constructor() {
        let actual = plugin_execute("plugin", vec!["execute".to_string()]);
        let expected = Step::Command(CommandStep::PluginExecute(PluginExecuteCommand {
            executable: "plugin".to_string(),
            args: vec!["execute".to_string()],
        }));
        assert_eq!(actual, expected);
    }
}
