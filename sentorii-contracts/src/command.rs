//! Defines all types related to a single, executable step in a workflow.

use super::step::StepCategory;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl SentoriiCommand {
    /// Returns the corresponding high-level category for the command.
    #[must_use]
    pub const fn category(&self) -> StepCategory {
        match self {
            Self::GitStatusCheck => StepCategory::Check,
            Self::GitCheckout { .. } | Self::GitCheckoutNewBranch { .. } => StepCategory::Checkout,
            Self::GitPull { .. } => StepCategory::Pull,
            Self::GitMergeNoFf { .. } => StepCategory::Merge,
            Self::GitPush { .. } | Self::GitPushTags => StepCategory::Push,
            Self::GitTag { .. } => StepCategory::Tag,
            Self::GitBranchDelete { .. } => StepCategory::DeleteBranch,
            Self::PluginExecute { .. } => StepCategory::Plugin,
        }
    }

    // --- Ergonomic Constructors ---

    pub fn git_checkout(branch: impl Into<String>) -> Self {
        Self::GitCheckout {
            branch: branch.into(),
        }
    }

    pub fn git_checkout_new_branch(branch: impl Into<String>) -> Self {
        Self::GitCheckoutNewBranch {
            branch: branch.into(),
        }
    }

    pub fn git_pull(remote: impl Into<String>, branch: impl Into<String>) -> Self {
        Self::GitPull {
            remote: remote.into(),
            branch: branch.into(),
        }
    }

    pub fn git_merge(branch: impl Into<String>) -> Self {
        Self::GitMergeNoFf {
            branch: branch.into(),
        }
    }

    pub fn git_push(remote: impl Into<String>, branch: impl Into<String>) -> Self {
        Self::GitPush {
            remote: remote.into(),
            branch: branch.into(),
        }
    }

    pub fn git_tag(tag: impl Into<String>) -> Self {
        Self::GitTag { tag: tag.into() }
    }

    pub fn git_delete_branch(branch: impl Into<String>) -> Self {
        Self::GitBranchDelete {
            branch: branch.into(),
        }
    }

    pub fn plugin_execute<E, A>(executable: E, args: A) -> Self
    where
        E: Into<String>,
        A: IntoIterator,
        A::Item: AsRef<str>,
    {
        Self::PluginExecute {
            executable: executable.into(),
            args: args.into_iter().map(|s| s.as_ref().to_string()).collect(),
        }
    }
}

impl fmt::Display for SentoriiCommand {
    /// Formats the command into its human-readable shell string representation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitStatusCheck => write!(f, "git status --porcelain"),
            Self::GitCheckout { branch } => write!(f, "git checkout {branch}"),
            Self::GitPull { remote, branch } => write!(f, "git pull {remote} {branch}"),
            Self::GitCheckoutNewBranch { branch } => write!(f, "git checkout -b {branch}"),
            Self::GitMergeNoFf { branch } => write!(f, "git merge --no-ff {branch}"),
            Self::GitPush { remote, branch } => write!(f, "git push {remote} {branch}"),
            Self::GitTag { tag } => write!(f, "git tag {tag}"),
            Self::GitPushTags => write!(f, "git push --tags"),
            Self::GitBranchDelete { branch } => write!(f, "git branch -d {branch}"),
            Self::PluginExecute { executable, args } => {
                let mut command_str = executable.clone();
                for arg in args {
                    command_str.push(' ');
                    if arg.contains(' ') {
                        command_str.push('"');
                        command_str.push_str(arg);
                        command_str.push('"');
                    } else {
                        command_str.push_str(arg);
                    }
                }
                write!(f, "{command_str}")
            }
        }
    }
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
