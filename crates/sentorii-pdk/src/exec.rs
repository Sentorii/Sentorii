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
pub fn stream_command(cmd: &mut Command) -> Result<std::process::ExitStatus, PdkError> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| PdkError::Io(e.into()))?;
    let (tx, rx) = mpsc::channel::<(LogLevel, String)>();

    let stdout_handle = spawn_reader_thread(child.stdout.take(), tx.clone(), LogLevel::Stdout);
    let stderr_handle = spawn_reader_thread(child.stderr.take(), tx, LogLevel::Stderr);

    for (level, message) in rx {
        log(level, message, LogSource::Process);
    }

    let status = child.wait().map_err(|e| PdkError::Io(e.into()))?;

    if let Some(handle) = stdout_handle {
        handle.join().expect("Stdout reader thread panicked");
    }
    if let Some(handle) = stderr_handle {
        handle.join().expect("Stderr reader thread panicked");
    }

    Ok(status)
}

/// Spawns a dedicated thread to read from a stream and send lines over a channel.
///
/// If the provided `stream` is `None`, it immediately sends a `Finished` message
/// and returns `None`.
fn spawn_reader_thread<R: Read + Send + 'static>(
    stream: Option<R>,
    tx: mpsc::Sender<(LogLevel, String)>,
    level: LogLevel,
) -> Option<JoinHandle<()>> {
    stream.map(|stream| {
        thread::spawn(move || {
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if tx.send((level.clone(), line)).is_err() {
                        break;
                    }
                }
            }
        })
    })
}
