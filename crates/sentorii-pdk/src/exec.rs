//! Helpers for executing external commands and streaming their output.

use crate::emitter::Emitter;
use crate::error::PdkError;
use sentorii_api::Stream;
use std::io::{BufRead, BufReader, Read, StdoutLock};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

/// An internal message sent from a reader thread to the main thread's event loop.
enum OutputLine {
    /// A line of content from a specific stream.
    Line(Stream, String),
    /// A signal that the reader thread has finished.
    Finished,
}

/// The public, high-level function to execute a command and stream its output.
///
/// This function coordinates the spawning of the child process, the creation of
/// reader threads, and the processing of the output event loop.
pub fn stream_command(
    cmd: &mut Command,
    emitter: &mut Emitter<StdoutLock>,
) -> Result<std::process::ExitStatus, PdkError> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| PdkError::Io(e.into()))?;
    let (tx, rx) = mpsc::channel::<OutputLine>();

    // Spawn threads to read stdout and stderr concurrently.
    let stdout_handle = spawn_reader_thread(child.stdout.take(), tx.clone(), Stream::Stdout);
    let stderr_handle = spawn_reader_thread(child.stderr.take(), tx, Stream::Stderr);

    // Run the main event loop on this thread, processing messages from the readers.
    process_output_loop(rx, emitter)?;

    // Wait for the child process and reader threads to complete.
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
    tx: mpsc::Sender<OutputLine>,
    stream_type: Stream,
) -> Option<JoinHandle<()>> {
    let stream = match stream {
        Some(s) => s,
        None => {
            // If the stream doesn't exist, signal that it's "finished" immediately.
            let _ = tx.send(OutputLine::Finished);
            return None;
        }
    };

    let handle = thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            if let Ok(line) = line {
                // If the channel is disconnected, the main thread has exited, so we stop.
                if tx.send(OutputLine::Line(stream_type.clone(), line)).is_err() {
                    break;
                }
            }
        }
        // Signal that this stream is finished.
        let _ = tx.send(OutputLine::Finished);
    });

    Some(handle)
}

/// Processes incoming `OutputLine` messages from the channel and dispatches to the emitter.
///
/// This loop runs until it receives a `Finished` message from both reader threads.
fn process_output_loop(
    rx: mpsc::Receiver<OutputLine>,
    emitter: &mut Emitter<StdoutLock>,
) -> Result<(), PdkError> {
    let mut finished_streams = 0;
    for received in rx {
        match received {
            OutputLine::Line(Stream::Stdout, content) => emitter.stdout(&content)?,
            OutputLine::Line(Stream::Stderr, content) => emitter.stderr(&content)?,
            OutputLine::Finished => {
                finished_streams += 1;
                if finished_streams == 2 {
                    break;
                }
            }
        }
    }
    Ok(())
}