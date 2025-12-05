//! Defines the abstract interface for executing commands.

use crate::command::SentoriiCommand;
use async_trait::async_trait;
use thiserror::Error;

/// Represents the final status of a command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

/// Custom error type for command execution failures.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum CommandExecutionError {
    /// Occurs when the underlying lock on internal state (e.g., for mocks) is poisoned.
    #[error("Mutex lock was poisoned")]
    LockPoisoned,
    /// A generic error that can be used for other execution failures.
    #[error("Command execution failed: {0}")]
    General(String),
}

/// The abstract interface for any struct that can execute a `SentoriiCommand`.
#[async_trait]
pub trait CommandRunner {
    /// Executes the given command.
    async fn execute(
        &self,
        command: SentoriiCommand,
    ) -> Result<ExecutionStatus, CommandExecutionError>;
}

// --- Mock Implementation for Testing ---

#[cfg(feature = "test_utils")]
use std::sync::{Arc, Mutex};

/// A mock implementation of the `CommandRunner` trait for use in tests.
#[cfg(feature = "test_utils")]
#[derive(Debug, Clone, Default)]
pub struct MockCommandRunner {
    /// A thread-safe log of all commands that were "executed"
    pub executed_commands: Arc<Mutex<Vec<SentoriiCommand>>>,
    /// If set, the runner will return `ExecutionStatus::Failure` when it encounters this command.
    pub should_fail_on: Option<SentoriiCommand>,
}

#[cfg(feature = "test_utils")]
#[async_trait]
impl CommandRunner for MockCommandRunner {
    async fn execute(
        &self,
        command: SentoriiCommand,
    ) -> Result<ExecutionStatus, CommandExecutionError> {
        self.executed_commands
            .lock()
            .map_err(|_| CommandExecutionError::LockPoisoned)?
            .push(command.clone());

        if let Some(fail_command) = &self.should_fail_on
            && &command == fail_command
        {
            return Ok(ExecutionStatus::Failure);
        }
        Ok(ExecutionStatus::Success)
    }
}

#[cfg(test)]
#[cfg(feature = "test_utils")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_runner_success() {
        let runner = MockCommandRunner::default();
        let command = SentoriiCommand::GitStatusCheck;

        let result = runner.execute(command.clone()).await;
        assert_eq!(result, Ok(ExecutionStatus::Success));

        match runner.executed_commands.lock() {
            Ok(executed) => {
                assert_eq!(executed.len(), 1);
                assert_eq!(executed[0], command);
            }
            Err(..) => panic!("Test failed: Mutex was poisoned unexpectedly."),
        }
    }

    #[tokio::test]
    async fn test_mock_runner_failure() {
        let fail_command = SentoriiCommand::git_pull("origin", "main");
        let runner = MockCommandRunner {
            should_fail_on: Some(fail_command.clone()),
            ..Default::default()
        };

        let status_ok = runner.execute(SentoriiCommand::GitStatusCheck).await;
        assert_eq!(status_ok, Ok(ExecutionStatus::Success));

        let status_fail = runner
            .execute(SentoriiCommand::git_pull("origin", "main"))
            .await;
        assert_eq!(status_fail, Ok(ExecutionStatus::Failure));

        match runner.executed_commands.lock() {
            Ok(executed) => {
                assert_eq!(executed.len(), 2);
            }
            Err(..) => panic!("Test failed: Mutex was poisoned unexpectedly."),
        }
    }

    #[tokio::test]
    async fn test_mock_handles_lock_poisoning() {
        let runner = MockCommandRunner::default();
        let runner_clone = runner.clone();

        let handle = std::thread::spawn(move || {
            let _lock = runner_clone.executed_commands.lock();
            panic!("Intentionally poisoning the lock for testing.");
        });

        assert!(handle.join().is_err(), "Thread did not panic as expected.");

        let result = runner.execute(SentoriiCommand::GitStatusCheck).await;

        assert_eq!(result, Err(CommandExecutionError::LockPoisoned));
    }
}
