//! Provides the `Emitter` struct for sending streaming responses to the host.

use std::io;
use std::io::Write;
use sentorii_api::{ProcessOutput, Response, Stream};

/// An emitter for sending streaming `Response` messages to the Sentorii host.
///
/// An instance of the emitter is passed to the plugin's logic handler, allowing
/// the plugin to send real-time `ProcessOutput` messages (for stdout/stderr of
/// child processes) or other intermediate data before the final `Success` or
/// `Error` response is sent.
pub struct Emitter<'a, W: Write> {
    writer: &'a mut W,
}

impl<'a, W: Write> Emitter<'a, W> {
    /// Creates a new `Emitter` that will write to the given writer.
    pub(crate) fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    /// Sends a line of content from a child process's stdout stream.
    ///
    /// This method serializes the content into a `Response::ProcessOutput` message
    /// and writes it immediately to the host.
    ///
    /// # Arguments
    /// * `content`: The string slice representing the line of output.
    pub fn stdout(&mut self, content: &str) -> io::Result<()> {
        self.emit_output(Stream::Stdout, content)
    }

    /// Sends a line of content from a child process's stderr stream.
    ///
    /// This method serializes the content into a `Response::ProcessOutput` message
    /// and writes it immediately to the host.
    ///
    /// # Arguments
    /// * `content`: The string slice representing the line of output.
    pub fn stderr(&mut self, content: &str) -> io::Result<()> {
        self.emit_output(Stream::Stderr, content)
    }

    fn emit_output(&mut self, stream: Stream, content: &str) -> io::Result<()> {
        let response = Response::ProcessOutput(ProcessOutput {
            stream,
            content: content.to_string(),
        });

        if let Ok(json) = serde_json::to_string(&response) {
            writeln!(self.writer, "{}", json)?;
            self.writer.flush()?;
        }
        Ok(())
    }
}