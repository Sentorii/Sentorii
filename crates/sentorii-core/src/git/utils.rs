//! Utility functions for interacting with the Git repository context.

use crate::error::{CoreError, InvalidStateError};
use sentorii_contracts::error::CommandExecutionError;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Finds the root directory of the current Git repository by calling `git rev-parse --show-toplevel`.
///
/// This is the definitive source of the repository root for the entire application.
/// It is called once at the beginning of each workflow dispatch.
///
/// # Errors
///
/// This function will return a `CoreError` if:
/// - The `git` command cannot be spawned (e.g., not in PATH).
/// - The current directory is not part of a Git repository.
/// - The output from the command is not valid UTF-8.
pub async fn resolve_git_root() -> Result<PathBuf, CoreError> {
    let command_str = "git rev-parse --show-toplevel";

    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            CoreError::CommandExecutionFailed(CommandExecutionError::Spawn {
                command: command_str.to_string(),
                source: e,
            })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::CommandExecutionFailed(
            CommandExecutionError::NonZeroStatus {
                command: command_str.to_string(),
                status: format!("'{}'", stderr.trim()),
            },
        ));
    }

    let path_str = String::from_utf8(output.stdout)
        .map_err(|_| CoreError::InvalidState(InvalidStateError::InvalidGitOutput))?;

    Ok(PathBuf::from(path_str.trim()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::process::Command as StdCommand;
    use tempfile::TempDir;

    #[cfg(feature = "test-integration")]
    #[tokio::test]
    async fn test_resolve_git_root_success() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path();

        assert!(
            StdCommand::new("git")
                .arg("init")
                .current_dir(repo_root)
                .status()
                .unwrap()
                .success()
        );

        let sub_dir = repo_root.join("src");
        tokio::fs::create_dir(&sub_dir).await.unwrap();
        std::env::set_current_dir(&sub_dir).unwrap();

        let result = resolve_git_root().await;

        assert!(result.is_ok());
        let found_root = result.unwrap();

        pretty_assertions::assert_eq!(
            found_root.canonicalize().unwrap(),
            repo_root.canonicalize().unwrap(),
        );
    }

    #[cfg(feature = "test-integration")]
    #[tokio::test]
    async fn test_resolve_git_root_failure_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = resolve_git_root().await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        assert!(matches!(
            error,
            CoreError::CommandExecutionFailed(CommandExecutionError::NonZeroStatus { .. })
        ));
    }
}
