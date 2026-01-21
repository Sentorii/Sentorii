#![forbid(unsafe_code)]

use crate::error::PdkError;
use sentorii_api::{
    ErrorCode, ErrorResponse, PluginInfo, Request, Response, SetVersionPayload, VersionResponse,
};
use std::io;
use std::io::{BufRead, StdoutLock, Write};
use std::process::exit;

pub mod error;
pub mod exec;
pub mod logging;

use crate::logging::error;
pub use sentorii_api;

pub trait Plugin {
    fn get_info(&mut self) -> Result<PluginInfo, PdkError>;
    fn get_version(&mut self) -> Result<VersionResponse, PdkError>;
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
        let line = match line {
            Ok(line) => line,
            Err(_) => return,
        };

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
    stdout.flush().expect("Failed to flush stdout");
}
