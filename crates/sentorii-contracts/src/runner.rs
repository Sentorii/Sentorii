//! Defines the abstract interface for executing commands.

use async_trait::async_trait;

/// Represents the final status of a command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failure,
}

/// The abstract interface for any struct that can execute a `SentoriiCommand`.
#[async_trait]
pub trait CommandRunner {
    /// Executes the given command.
    async fn execute(&self, command: ExecutableCommand, step_id: usize) -> Result<(), CommandExecutionError>;
}

// --- Mock Implementation for Testing ---

use crate::command::ExecutableCommand;
use crate::error::CommandExecutionError;
#[cfg(feature = "test_utils")]
use std::sync::{Arc, Mutex};

/// A mock implementation of the `CommandRunner` trait for use in tests.
#[cfg(feature = "test_utils")]
#[derive(Debug, Clone, Default)]
pub struct MockCommandRunner {
    /// A thread-safe log of all commands that were "executed"
    pub executed_commands: Arc<Mutex<Vec<ExecutableCommand>>>,
    /// If set, the runner will return `ExecutionStatus::Failure` when it encounters this command.
    pub should_fail_on: Option<ExecutableCommand>,
}

#[cfg(feature = "test_utils")]
#[async_trait]
impl CommandRunner for MockCommandRunner {
    async fn execute(&self, command: ExecutableCommand, _step_id: usize) -> Result<(), CommandExecutionError> {
        self.executed_commands
            .lock()
            .map_err(|_| CommandExecutionError::LockPoisoned)?
            .push(command.clone());

        if let Some(fail_command) = &self.should_fail_on
            && &command == fail_command
        {
            return Err(CommandExecutionError::NonZeroStatus {
                command: command.command(),
                status: "Failed".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "test_utils")]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::command::{Command, GitCheckOutCommand, GitStatusCheckCommand};
    use crate::context::{ContextBuilder, ValueSource};
    use crate::step::CommandStep;

    #[tokio::test]
    async fn test_mock_runner_success() {
        let runner = MockCommandRunner::default();
        let context = ContextBuilder::new().build();
        let command = CommandStep::GitStatusCheck(GitStatusCheckCommand)
            .to_executable(&context)
            .unwrap();

        let result = runner.execute(command.clone(), 1).await;
        assert!(matches!(result, Ok(())));

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
        let context = ContextBuilder::new().build();
        let fail_command = CommandStep::GitCheckout(GitCheckOutCommand {
            branch: ValueSource::Literal("fail".to_string()),
        })
        .to_executable(&context)
        .unwrap();
        let runner = MockCommandRunner {
            should_fail_on: Some(fail_command.clone()),
            ..Default::default()
        };

        let status_ok = runner
            .execute(
                CommandStep::GitStatusCheck(GitStatusCheckCommand)
                    .to_executable(&context)
                    .unwrap(),
                1
            )
            .await;
        assert!(matches!(status_ok, Ok(())));

        let status_fail = runner
            .execute(
                CommandStep::GitCheckout(GitCheckOutCommand {
                    branch: ValueSource::Literal("fail".to_string()),
                })
                .to_executable(&context)
                .unwrap(),
            2)
            .await;
        assert!(matches!(
            status_fail,
            Err(CommandExecutionError::NonZeroStatus { .. })
        ));

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
        let context = ContextBuilder::new().build();
        let runner_clone = runner.clone();

        let handle = std::thread::spawn(move || {
            let _lock = runner_clone.executed_commands.lock();
            panic!("Intentionally poisoning the lock for testing.");
        });

        assert!(handle.join().is_err(), "Thread did not panic as expected.");

        let result = runner
            .execute(
                CommandStep::GitStatusCheck(GitStatusCheckCommand)
                    .to_executable(&context)
                    .unwrap(),
                1
            )
            .await;

        assert!(matches!(result, Err(CommandExecutionError::LockPoisoned)));
    }
}
