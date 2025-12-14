//! Defines all types related to a single, executable step in a workflow.

use super::step::Category;
use crate::context::Context;
use crate::error::CommandBuildError;
use crate::event::{RecoveryAction, RevertAction};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A low-level, executable command ready to be executed by a `CommandRunner`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableCommand {
    pub program: String,
    pub args: Vec<String>,
}

/// Contract for all self-contained command steps.
pub trait Command: Debug {
    /// Provides a human-readable description for the UI.
    fn description(&self) -> String;
    /// Provides a category for UI grouping and icons.
    fn category(&self) -> Category;
    /// Translates the step into a low-level command, resolving placeholders.
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError>;
    /// Provides a list of possible actions to revert this step's effects.
    fn possible_reverts(&self) -> Vec<RevertAction>;
    /// Provides a list of possible actions to recover from a failure of this step.
    fn possible_recoveries(&self) -> Vec<RecoveryAction>;
}

/// Defines the `CommandStep` enum and automatically implements the `Command` trait for it.
///
/// This macro is the single source of truth for all commands in the system.
/// To add a new command:
/// 1. Create a new struct (e.g., `MyNewCommand`).
/// 2. Implement the `Command` trait for `MyNewCommand`.
/// 3. Add one line to this macro invocation: `MyNew(MyNewCommand)`.
/// 4. (Optional) Create a top-level helper function `pub fn my_new(...) -> Step`.
macro_rules! command_step {
    ( $( $(#[$variant_meta:meta])* $variant:ident($command_type:ty) ),* ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub enum CommandStep {
            $( $(#[$variant_meta])* $variant($command_type), )*
        }

        impl Command for CommandStep {
            fn description(&self) -> String {
                match self { $( Self::$variant(cmd) => cmd.description(), )* }
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
    };
}

command_step!(GitPull(GitPullCommand));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitPullCommand {
    pub remote: String,
    pub branch: String,
}

impl Command for GitPullCommand {
    fn description(&self) -> String {
        format!(
            "Pulling branch '{}' from remote '{}'",
            &self.branch, &self.remote
        )
    }
    fn category(&self) -> Category {
        Category::Pull
    }
    fn to_executable(&self, _context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        Ok(ExecutableCommand {
            program: "git".to_string(),
            args: vec!["pull".to_string(), self.branch.clone()],
        })
    }
    fn possible_reverts(&self) -> Vec<RevertAction> {
        vec![]
    }
    fn possible_recoveries(&self) -> Vec<RecoveryAction> {
        vec![]
    }
}

/// A detailed, type-safe representation of every possible command the engine can execute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SentoriiCommand {
    GitStatusCheck,
    GitCheckout {
        branch: String,
    },
    GitPull {
        remote: String,
        branch: String,
    },
    GitCheckoutNewBranch {
        branch: String,
    },
    GitMergeNoFf {
        branch: String,
    },
    GitPush {
        remote: String,
        branch: String,
    },
    GitTag {
        tag: String,
    },
    GitPushTags,
    GitBranchDelete {
        branch: String,
    },
    PluginExecute {
        executable: String,
        args: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_checkout_constructor() {
        let actual = SentoriiCommand::git_checkout("awesome-feature");
        let expected = SentoriiCommand::GitCheckout {
            branch: "awesome-feature".into(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_checkout_new_branch_constructor() {
        let actual = SentoriiCommand::git_checkout_new_branch("awesome-feature");
        let expected = SentoriiCommand::GitCheckoutNewBranch {
            branch: "awesome-feature".into(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_pull_constructor() {
        let actual = SentoriiCommand::git_pull("origin", "awesome-feature");
        let expected = SentoriiCommand::GitPull {
            remote: "origin".into(),
            branch: "awesome-feature".into(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_merge_constructor() {
        let actual = SentoriiCommand::git_merge("awesome-feature");
        let expected = SentoriiCommand::GitMergeNoFf {
            branch: "awesome-feature".into(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_constructor() {
        let actual = SentoriiCommand::git_push("origin", "awesome-feature");
        let expected = SentoriiCommand::GitPush {
            remote: "origin".into(),
            branch: "awesome-feature".into(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_tag_constructor() {
        let actual = SentoriiCommand::git_tag("v1");
        let expected = SentoriiCommand::GitTag { tag: "v1".into() };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_delete_branch_constructor() {
        let actual = SentoriiCommand::git_delete_branch("awesome-feature");
        let expected = SentoriiCommand::GitBranchDelete {
            branch: "awesome-feature".into(),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_execute_plugin_constructor() {
        let actual = SentoriiCommand::plugin_execute("cargo", &["build", "--release"]);
        let expected = SentoriiCommand::PluginExecute {
            executable: "cargo".to_string(),
            args: vec!["build".to_string(), "--release".to_string()],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_status_check_display() {
        assert_eq!(
            format!("{}", SentoriiCommand::GitStatusCheck),
            "git status --porcelain"
        );
    }

    #[test]
    fn test_git_checkout_display() {
        let actual = format!("{}", SentoriiCommand::git_checkout("awesome-feature"));
        let expected = "git checkout awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_pull_display() {
        let actual = format!("{}", SentoriiCommand::git_pull("origin", "awesome-feature"));
        let expected = "git pull origin awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_checkout_new_branch_display() {
        let actual = format!(
            "{}",
            SentoriiCommand::git_checkout_new_branch("awesome-feature")
        );
        let expected = "git checkout -b awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_merge_display() {
        let actual = format!("{}", SentoriiCommand::git_merge("awesome-feature"));
        let expected = "git merge --no-ff awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_display() {
        let actual = format!("{}", SentoriiCommand::git_push("origin", "awesome-feature"));
        let expected = "git push origin awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_tag_display() {
        let actual = format!("{}", SentoriiCommand::git_tag("v1"));
        let expected = "git tag v1";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_tags_display() {
        let actual = format!("{}", SentoriiCommand::GitPushTags);
        let expected = "git push --tags";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_delete_branch_display() {
        let actual = format!("{}", SentoriiCommand::git_delete_branch("awesome-feature"));
        let expected = "git branch -d awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_execute_plugin_display() {
        let actual = format!(
            "{}",
            SentoriiCommand::plugin_execute("cargo", &["build", "--release"])
        );
        let expected = "cargo build --release";
        assert_eq!(actual, expected);
    }
}
