//! Defines all types related to a single, executable step in a workflow.

use super::step::Category;
use crate::context::{Context, ValueSource};
use crate::error::CommandBuildError;
use crate::event::{RecoveryAction, RevertAction};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug};

/// A low-level, executable command ready to be executed by a `CommandRunner`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl ExecutableCommand {
    pub fn git<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            program: "git".to_string(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn new<I, S>(program: &str, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            program: program.to_string(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn command(&self) -> String
    {
        format!("{} {}", self.program, self.args.join(" "))
    }
}

/// Contract for all self-contained command steps.
pub trait Command: Debug {
    /// Provides a human-readable description for the UI.
    fn static_description(&self) -> String;
    /// Returns a specific, resolved description for more context.
    fn resolved_description(&self, _context: &Context) -> String {
        self.static_description()
    }
    /// Provides a category for UI grouping and icons.
    fn category(&self) -> Category;
    /// Translates the step into a low-level command, resolving placeholders.
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError>;
    /// Provides a list of possible actions to revert this step's effects.
    fn possible_reverts(&self) -> Vec<RevertAction> {
        vec![]
    }
    /// Provides a list of possible actions to recover from a failure of this step.
    fn possible_recoveries(&self) -> Vec<RecoveryAction> {
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatusCheckCommand;

impl Command for GitStatusCheckCommand {
    fn static_description(&self) -> String {
        "Check if status is clean".to_string()
    }
    fn category(&self) -> Category {
        Category::Check
    }
    fn to_executable(&self, _context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        Ok(ExecutableCommand::git(["status", "--porcelain"]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCheckOutCommand {
    pub branch: ValueSource,
}

impl Command for GitCheckOutCommand {
    fn static_description(&self) -> String {
        "Check out new branch".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match self.branch.resolve(context) {
            Ok(resolved_branch) => {
                format!("Check out branch {resolved_branch}")
            }
            Err(_) => {
                self.static_description()
            }
        }
    }
    fn category(&self) -> Category {
        Category::Checkout
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_branch = self.branch.resolve(context)?;
        Ok(ExecutableCommand::git(["checkout", &resolved_branch]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCheckoutNewBranchCommand {
    pub(crate) branch: ValueSource,
}

impl Command for GitCheckoutNewBranchCommand {
    fn static_description(&self) -> String {
        "Checkout new branch".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match self.branch.resolve(context) {
            Ok(resolved_branch) => {
                format!("Checkout new branch {resolved_branch}")
            }
            Err(_) => {
                self.static_description()
            }
        }
    }
    fn category(&self) -> Category {
        Category::Checkout
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_branch = self.branch.resolve(context)?;
        Ok(ExecutableCommand::git(["checkout", "-b", &resolved_branch]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitPullCommand {
    pub remote: ValueSource,
    pub branch: ValueSource,
}

impl Command for GitPullCommand {
    fn static_description(&self) -> String {
        "Pull branch".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match (self.remote.resolve(context), self.branch.resolve(context)) {
            (Ok(remote), Ok(branch)) => {
                format!("Pull from {remote}/{branch}")
            }
            _ => self.static_description(),
        }
    }
    fn category(&self) -> Category {
        Category::Pull
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_remote = self.remote.resolve(context)?;
        let resolved_branch = self.branch.resolve(context)?;
        Ok(ExecutableCommand::git(["pull", &resolved_remote, &resolved_branch]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitMergeNoFfCommand {
    pub(crate) branch: ValueSource,
}

impl Command for GitMergeNoFfCommand {
    fn static_description(&self) -> String {
        "Merge branches".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match self.branch.resolve(context) {
            Ok(resolved_branch) => {
                format!("Merge branch {resolved_branch}")
            }
            Err(_) => {
                self.static_description()
            }
        }
    }
    fn category(&self) -> Category {
        Category::Merge
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_branch = self.branch.resolve(context)?;
        Ok(ExecutableCommand::git(["merge", "--no-ff", &resolved_branch]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitPushCommand {
    pub(crate) remote: ValueSource,
    pub(crate) branch: ValueSource,
}

impl Command for GitPushCommand {
    fn static_description(&self) -> String {
        "Push branch".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match (self.remote.resolve(context), self.branch.resolve(context)) {
            (Ok(remote), Ok(branch)) => {
                format!("Push {branch} to {remote}")
            }
            _ => self.static_description(),
        }
    }
    fn category(&self) -> Category {
        Category::Push
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_remote = self.remote.resolve(context)?;
        let resolved_branch = self.branch.resolve(context)?;
        Ok(ExecutableCommand::git(["push", &resolved_remote, &resolved_branch]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitTagCommand {
    pub(crate) tag: ValueSource,
}

impl Command for GitTagCommand {
    fn static_description(&self) -> String {
        "Create a new tag".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match self.tag.resolve(context) {
            Ok(resolved_tag) => {
                format!("Tag {resolved_tag}")
            }
            Err(_) => {
                self.static_description()
            }
        }
    }
    fn category(&self) -> Category {
        Category::Tag
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_tag = self.tag.resolve(context)?;
        Ok(ExecutableCommand::git(["tag", &resolved_tag]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitPushTagsCommand;

impl Command for GitPushTagsCommand {
    fn static_description(&self) -> String {
        "Push tags".to_string()
    }
    fn category(&self) -> Category {
        Category::Push
    }
    fn to_executable(&self, _context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        Ok(ExecutableCommand::git(["push", "--tags"]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitBranchDeleteCommand {
    pub(crate) branch: ValueSource,
}

impl Command for GitBranchDeleteCommand {
    fn static_description(&self) -> String {
        "Delete branch".to_string()
    }
    fn resolved_description(&self, context: &Context) -> String {
        match self.branch.resolve(context) {
            Ok(resolved_branch) => {
                format!("Delete branch {resolved_branch}")
            }
            Err(_) => {
                self.static_description()
            }
        }
    }
    fn category(&self) -> Category {
        Category::DeleteBranch
    }
    fn to_executable(&self, context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        let resolved_branch = self.branch.resolve(context)?;
        Ok(ExecutableCommand::git(["branch", "-d", &resolved_branch]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginExecuteCommand {
    pub(crate) executable: String,
    pub(crate) args: Vec<String>,
}

impl Command for PluginExecuteCommand {
    fn static_description(&self) -> String {
        self.executable.clone()
    }
    fn category(&self) -> Category {
        Category::Plugin
    }
    fn to_executable(&self, _context: &Context) -> Result<ExecutableCommand, CommandBuildError> {
        Ok(ExecutableCommand::new(&*self.executable, self.args.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{ContextBuilder, ContextKey};
    use super::*;

    #[test]
    fn test_git_status_check_executable() {
        let context = ContextBuilder::new().build();
        let actual = GitStatusCheckCommand.to_executable(&context).unwrap().command();
        let expected = "git status --porcelain";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_checkout_executable() {
        let command = GitCheckOutCommand {
            branch: ValueSource::FromContext(ContextKey::Develop)
        };
        let context = ContextBuilder::new().with_develop("test").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git checkout test";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_checkout_new_branch_executable() {
        let command = GitCheckoutNewBranchCommand {
            branch: ValueSource::FromContext(ContextKey::FeatureBranch)
        };
        let context = ContextBuilder::new().with_feature_branch("feature/awesome-feature").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git checkout -b feature/awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_pull_executable() {
        let command = GitPullCommand {
            remote: ValueSource::FromContext(ContextKey::Remote),
            branch: ValueSource::FromContext(ContextKey::FeatureBranch)
        };
        let context = ContextBuilder::new().with_remote("origin").with_feature_branch("feature/awesome-feature").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git pull origin feature/awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_merge_executable() {
        let command = GitMergeNoFfCommand {
            branch: ValueSource::FromContext(ContextKey::FeatureBranch)
        };
        let context = ContextBuilder::new().with_feature_branch("feature/awesome-feature").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git merge --no-ff feature/awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_executable() {
        let command = GitPushCommand {
            remote: ValueSource::FromContext(ContextKey::Remote),
            branch: ValueSource::FromContext(ContextKey::FeatureBranch)
        };
        let context = ContextBuilder::new().with_remote("origin").with_feature_branch("feature/awesome-feature").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git push origin feature/awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_tag_executable() {
        let command = GitTagCommand {
            tag: ValueSource::FromContext(ContextKey::Tag)
        };
        let context = ContextBuilder::new().with_tag("v1").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git tag v1";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_delete_branch_executable() {
        let command = GitBranchDeleteCommand {
            branch: ValueSource::FromContext(ContextKey::FeatureBranch)
        };
        let context = ContextBuilder::new().with_feature_branch("feature/awesome-feature").build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "git branch -d feature/awesome-feature";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_execute_plugin_executable() {
        let command = PluginExecuteCommand {
            executable: "plugin".to_string(),
            args: vec!["execute".to_string()],
        };
        let context = ContextBuilder::new().build();
        let executable = command.to_executable(&context).unwrap();
        let actual = executable.command();
        let expected = "plugin execute";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_git_push_tags_executable() {
        let context = ContextBuilder::new().build();
        let actual = GitPushTagsCommand.to_executable(&context).unwrap().command();
        let expected = "git push --tags";
        assert_eq!(actual, expected);
    }
}
