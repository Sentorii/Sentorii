//! Executes shell commands using `tokio::process` and streams I/O as events.

use async_trait::async_trait;
use sentorii_contracts::command::ExecutableCommand;
use sentorii_contracts::error::CommandExecutionError;
use sentorii_contracts::event::{Event, LogLine};
use sentorii_contracts::runner::CommandRunner;
use std::process::Stdio;
use log::Log;
use tokio::io::AsyncBufRead;
use tokio::process::Child;
use tokio::task::JoinHandle;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command as TokioCommand,
    sync::mpsc,
};

/// Executes `ExecutableCommand` instances as real system processes.
#[derive(Debug, Clone)]
pub struct CommandExecutor {
    event_tx: mpsc::Sender<Event>,
}

impl CommandExecutor {
    pub const fn new(event_tx: mpsc::Sender<Event>) -> Self {
        Self { event_tx }
    }

    /// Spawns a child process for the given command.
    fn spawn_child(
        command: &ExecutableCommand,
        full_command_str: &str,
    ) -> Result<Child, CommandExecutionError> {
        TokioCommand::new(&command.program)
            .args(&command.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CommandExecutionError::Spawn {
                command: full_command_str.to_string(),
                source: e,
            })
    }

    /// Spawns a dedicated task to stream lines from a reader, tagging them with the correct stream type.
    fn stream_lines<R: AsyncBufRead + Unpin + Send + 'static>(
        reader: R,
        log_line: fn(String) -> LogLine,
        step_id: usize,
        event_tx: mpsc::Sender<Event>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if event_tx
                    .send(Event::LogOutput{
                        step_id,
                        line: log_line(line)
                    })
                    .await
                    .is_err()
                {
                    break;
                }
            }
        })
    }
}

#[async_trait]
impl CommandRunner for CommandExecutor {
    async fn execute(&self, command: ExecutableCommand, step_id: usize) -> Result<(), CommandExecutionError> {
        let full_command_str = format!("{} {}", command.program, command.args.join(" "));

        let mut child = Self::spawn_child(&command, &full_command_str)?;

        let stdout = child
            .stdout
            .take()
            .ok_or(CommandExecutionError::TaskPanic)?;
        let stderr = child
            .stderr
            .take()
            .ok_or(CommandExecutionError::TaskPanic)?;

        let stdout_task = Self::stream_lines(
            BufReader::new(stdout),
            LogLine::Stdout,
            step_id,
            self.event_tx.clone(),
        );
        let stderr_task = Self::stream_lines(
            BufReader::new(stderr),
            LogLine::Stderr,
            step_id,
            self.event_tx.clone(),
        );
        let wait_task = tokio::spawn(async move {
            child.wait().await.map_err(|e| CommandExecutionError::Wait {
                command: full_command_str.clone(),
                source: e,
            })
        });

        let (wait_result, (), ()) = tokio::try_join!(wait_task, stdout_task, stderr_task)
            .map_err(|_| CommandExecutionError::TaskPanic)?;

        let status = wait_result?;

        if !status.success() {
            return Err(CommandExecutionError::NonZeroStatus {
                command: format!("{} {}", command.program, command.args.join(" ")),
                status: status.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "test-integration")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_success_and_streams_stdout() {
        let (event_tx, mut event_rx) = mpsc::channel::<Event>(100);
        let executor = CommandExecutor::new(event_tx);
        let command = ExecutableCommand::new("echo", ["hello stdout"]);

        let result = executor.execute(command).await;
        assert!(result.is_ok());

        let mut logs = Vec::new();
        while let Ok(event) = event_rx.try_recv() {
            if let Event::LogOutput { stream, line } = event {
                logs.push((stream, line));
            }
        }

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].0, LogStream::Stdout);
        assert!(logs[0].1.contains("hello stdout"));
    }

    #[tokio::test]
    async fn test_executor_success_and_streams_stderr() {
        let (event_tx, mut event_rx) = mpsc::channel::<Event>(100);
        let executor = CommandExecutor::new(event_tx);

        let command = if cfg!(windows) {
            ExecutableCommand::new("cmd", ["/C", "echo hello stderr 1>&2"])
        } else {
            ExecutableCommand::new("sh", ["-c", "echo 'hello stderr' >&2"])
        };

        let result = executor.execute(command).await;
        assert!(result.is_ok());

        let mut logs = Vec::new();
        while let Ok(event) = event_rx.try_recv() {
            if let Event::LogOutput { stream, line } = event {
                logs.push((stream, line));
            }
        }

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].0, LogStream::Stderr);
        assert!(logs[0].1.contains("hello stderr"));
    }
}
