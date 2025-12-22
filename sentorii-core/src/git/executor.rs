//! Executes shell commands using `tokio::process` and streams I/O as events.

use async_trait::async_trait;
use sentorii_contracts::command::ExecutableCommand;
use sentorii_contracts::error::CommandExecutionError;
use sentorii_contracts::event::Event;
use sentorii_contracts::runner::CommandRunner;
use std::process::Stdio;
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
}

#[async_trait]
impl CommandRunner for CommandExecutor {
    async fn execute(&self, command: ExecutableCommand) -> Result<(), CommandExecutionError> {
        let full_command_str = format!("{} {}", command.program, command.args.join(" "));

        let mut child = TokioCommand::new(&command.program)
            .args(&command.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CommandExecutionError::Spawn {
                command: full_command_str.clone(),
                source: e,
            })?;

        let stdout = BufReader::new(child.stdout.take().expect("stdout not captured"));
        let stderr = BufReader::new(child.stderr.take().expect("stderr not captured"));

        let tx_out = self.event_tx.clone();
        let stdout_task = tokio::spawn(async move {
            let mut reader = stdout.lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_out.send(Event::LogOutput(line)).await.is_err() {
                    break;
                }
            }
        });

        let tx_err = self.event_tx.clone();
        let stderr_task = tokio::spawn(async move {
            let mut reader = stderr.lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_err.send(Event::LogOutput(line)).await.is_err() {
                    break;
                }
            }
        });

        let status = child
            .wait()
            .await
            .map_err(|e| CommandExecutionError::Wait {
                command: full_command_str.clone(),
                source: e,
            })?;
        stdout_task
            .await
            .map_err(|_| CommandExecutionError::TaskPanic)?;
        stderr_task
            .await
            .map_err(|_| CommandExecutionError::TaskPanic)?;

        if !status.success() {
            return Err(CommandExecutionError::NonZeroStatus {
                command: full_command_str,
                status: status.to_string(),
            });
        }

        Ok(())
    }
}
