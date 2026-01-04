#![forbid(unsafe_code)]

use std::io;
use std::io::{BufRead, StdoutLock, Write};
use sentorii_api::{Request, Response, SuccessResponse};
use crate::error::PdkError;

pub mod error;
pub mod logging;
pub mod emitter;
mod exec;

pub use emitter::Emitter;

/// Runs the main plugin event loop.
///
/// This is the primary entry point for a Sentorii plugin. It handles all
/// communication with the Sentorii host application over stdin/stdout.
///
/// # Panics
/// ... (panic docs unchanged)
///
/// # Arguments
///
/// * `handler`: A closure that takes a `sentorii_api::Request` and a mutable
///   reference to an `Emitter` wrapping `stdout`. It is called for each incoming
///   request and must return a `Result` to signal completion of the command.
///
/// # Example
///
/// ```no_run
/// use sentorii_pdk::{run_plugin, Emitter};
/// use sentorii_pdk::error::PdkError;
/// use sentorii_api::{Request, PluginInfo, SuccessResponse};
/// use std::io::StdoutLock;
///
/// // The handler's signature must match what `run_plugin` provides.
/// fn my_plugin_logic(
///     request: Request,
///     emitter: &mut Emitter<StdoutLock>,
/// ) -> Result<SuccessResponse, PdkError> {
///     emitter.stdout("Received a request!").unwrap();
///     // ... logic
///     # Ok(SuccessResponse::Ack)
/// }
///
/// fn main() {
///     run_plugin(my_plugin_logic);
/// }
/// ```
pub fn run_plugin<F>(mut handler: F)
where
    F: FnMut(Request, &mut Emitter<StdoutLock>) -> Result<SuccessResponse, PdkError>,
{
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    for line in stdin.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => return,
        };

        if line.trim().is_empty() {
            continue;
        }

        let final_response = {
            let mut emitter = Emitter::new(&mut stdout);
            match serde_json::from_str::<Request>(&line) {
                Ok(request) => {
                    match handler(request, &mut emitter) {
                        Ok(success_data) => Response::Success(success_data),
                        Err(plugin_error) => Response::Error(plugin_error.into()),
                    }
                }
                Err(e) => Response::Error(PdkError::Json(e).into()),
            };
        };

        if let Ok(response_json) = serde_json::to_string(&final_response) {
            if writeln!(stdout, "{}", response_json).is_err() {
                return;
            };
        }

        stdout.flush().expect("Failed to flush stdout");
    }
}