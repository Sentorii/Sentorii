#![forbid(unsafe_code)]

use crate::error::PdkError;
use crate::logging::error;
use sentorii_api::{
    ErrorCode, ErrorResponse, PluginInfo, Request, Response, SetVersionPayload, VersionResponse,
};
use std::io;
use std::io::{BufRead, StdoutLock, Write};
use std::process::exit;

pub mod error;
pub mod exec;
pub mod logging;
pub use sentorii_api;

/// The central trait that all Sentorii plugins must implement.
///
/// This trait defines the core logic required by the Sentorii host. By implementing
/// these three methods, your plugin provides all the necessary functionality for
/// discovery, version reading, and version writing.
pub trait Plugin {
    /// Called when the host requests the plugin's static information.
    ///
    /// This method should return a `PluginInfo` struct describing the plugin's
    /// capabilities, name, and the files it operates on.
    ///
    /// # Errors
    /// Can crash on a range of `PdkError` specified by the plugin itself.
    fn get_info(&mut self) -> Result<PluginInfo, PdkError>;

    /// Called when the host wants to read the version from the project in the
    /// current working directory.
    ///
    /// # Errors
    /// Can crash on a range of `PdkError` specified by the plugin itself.
    fn get_version(&mut self) -> Result<VersionResponse, PdkError>;

    /// Called when the host wants to write a new version to the project in the
    /// current working directory.
    ///
    /// # Errors
    /// Can crash on a range of `PdkError` specified by the plugin itself.
    fn set_version(&mut self, payload: SetVersionPayload) -> Result<(), PdkError>;
}

pub fn run_plugin_with_init<P, F, E>(mut loader: F)
where
    P: Plugin,
    F: FnMut() -> Result<P, E>,
    E: Into<PdkError>,
{
    match loader() {
        Ok(plugin) => {
            run_plugin(plugin);
        }
        Err(loader_error) => {
            let pdk_error = loader_error.into();
            let error_response = Response::Error(pdk_error.into());

            let mut stdout = io::stdout().lock();
            send_final_response(&error_response, &mut stdout);
            exit(1);
        }
    }
}

pub fn run_plugin<P: Plugin>(mut plugin: P) {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    for line in stdin.lines() {
        let Ok(line) = line else { return };

        if line.trim().is_empty() {
            continue;
        }

        let final_response = {
            match serde_json::from_str::<Request>(&line) {
                Ok(request) => dispatch_request(&mut plugin, request),
                Err(e) => Response::Error(PdkError::Json(e).into()),
            }
        };

        send_final_response(&final_response, &mut stdout);
    }
}

fn dispatch_request<P: Plugin>(plugin: &mut P, request: Request) -> Response {
    match request {
        Request::GetInfo => match plugin.get_info() {
            Ok(info) => Response::Info(info),
            Err(e) => Response::Error(e.into()),
        },
        Request::GetVersion => match plugin.get_version() {
            Ok(version) => Response::Version(version),
            Err(e) => Response::Error(e.into()),
        },
        Request::SetVersion(payload) => match plugin.set_version(payload) {
            Ok(()) => Response::Ack,
            Err(e) => Response::Error(e.into()),
        },
    }
}

fn send_final_response(response: &Response, stdout: &mut StdoutLock) {
    match serde_json::to_string(response) {
        Ok(response_json) => {
            if writeln!(stdout, "{response_json}").is_err() {
                return;
            }
        }
        Err(e) => {
            error(&format!("Failed to serialize json: {e}"));
            let fallback = Response::Error(ErrorResponse {
                code: ErrorCode::PluginLogicFailed,
                message: "Internal plugin error: failed to serialize response.".to_string(),
            });
            if let Ok(fallback_json) = serde_json::to_string(&fallback) {
                let _ = writeln!(stdout, "{fallback_json}");
            }
        }
    }
    let _ = stdout.flush();
}
