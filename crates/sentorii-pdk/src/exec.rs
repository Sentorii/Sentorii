//! Helpers for executing external commands and streaming their output.

use crate::error::PdkError;
use crate::logging::log;
use sentorii_api::logging::{LogLevel, LogSource};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

/// The public, high-level function to execute a command and stream its output.
///
/// This function coordinates the spawning of the child process, the creation of
/// reader threads, and the processing of the output event loop.
///
/// # Errors
/// Can occur when process fails to spawn, timeouts occur, and when stdout or stderr is not closed as expected.
pub fn stream_command(cmd: &mut Command) -> Result<std::process::ExitStatus, PdkError> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let (tx, rx) = mpsc::channel::<(LogLevel, String)>();

    let stdout_handle = child
        .stdout
        .take()
        .map(|s| spawn_reader_thread(s, tx.clone(), LogLevel::Stdout));
    let stderr_handle = child
        .stderr
        .take()
        .map(|s| spawn_reader_thread(s, tx, LogLevel::Stderr));

    for (level, message) in rx {
        log(level, message, LogSource::Process);
    }

    let status = child.wait()?;

    if let Some(handle) = stdout_handle {
        handle
            .join()
            .map_err(|e| PdkError::PluginLogic(format!("Failed to close stdout. {e:?}")))?;
    }
    if let Some(handle) = stderr_handle {
        handle
            .join()
            .map_err(|e| PdkError::PluginLogic(format!("Failed to close stderr. {e:?}")))?;
    }

    Ok(status)
}

/// Spawns a dedicated thread to read from a stream and send lines over a channel.
///
/// If the provided `stream` is `None`, it immediately sends a `Finished` message
/// and returns `None`.
fn spawn_reader_thread<R: Read + Send + 'static>(
    stream: R,
    tx: mpsc::Sender<(LogLevel, String)>,
    level: LogLevel,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if tx.send((level, line)).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    })
}
