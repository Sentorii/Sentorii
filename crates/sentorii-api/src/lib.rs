#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// =========================================================================
// Top-Level Communication Enums (for line-delimited JSON protocol)
// =========================================================================

/// Represents a single request sent from the Sentorii host to a plugin.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "command", content = "payload", rename_all = "kebab-case")]
pub enum Request {
    /// Asks the plugin for its static information and capabilities.
    GetInfo,
    /// Asks the plugin to read and return the current version from the project.
    GetVersion,
    /// Commands the plugin to write a new version to the project files.
    SetVersion(SetVersionPayload),
}

/// Represents a single response sent from a plugin back to the Sentorii host.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "status", content = "data", rename_all = "kebab-case")]
pub enum Response {
    /// A successful response containing the requested data.
    Success(SuccessResponse),
    /// A response indicating a failure, with structured error information.
    Error(ErrorResponse),
    /// An intermediate response relaying output from a child proces or internal operation.
    ProcessOutput(ProcessOutput),
}

/// Contains the payload for a successful command execution.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum SuccessResponse {
    /// The response to a `get-info` request.
    Info(PluginInfo),
    /// The response to a `get-version` request.
    Version(VersionResponse),
    /// A generic acknowledgement for operations that don't return data, like `set-version`.
    Ack,
}

/// The payload for a `process-output` streaming message.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    /// The stream this output originated from.
    pub stream: Stream,
    /// The line of content.
    pub content: String,
}

/// Identifies the stream source for a `ProcessOutput` message.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Stream {
    Stdout,
    Stderr,
}


// =========================================================================
// Command-Specific Payloads and Responses
// =========================================================================

/// The payload for the `set-version` command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetVersionPayload {
    /// The new version string to write.
    pub version: String,
}

/// The successful data response for a `get-version` command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VersionResponse {
    /// The current version string read from the project.
    pub version: String,
}

/// The successful data response for a `get-info` command.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// The canonical, human-readable name of the plugin (e.g., "Cargo").
    pub plugin_name: String,
    /// The name of the plugin's executable binary (e.g., "sentorii-plugin-cargo").
    pub binary_name: String,
    /// The primary file this plugin looks for to detect a relevant project (e.g., "Cargo.toml").
    pub primary_file: String,
    /// A list of other files that suggest a project's presence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hint_files: Vec<String>,
    /// Security permissions required by the plugin.
    /// The host can use this to create a sandbox environment.
    #[serde(default)]
    pub permissions: PluginPermissions,
}

// =========================================================================
// Shared Structures (Errors and Permissions)
// =========================================================================

/// Defines the security permissions a plugin requires to operate.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct PluginPermissions {
    /// A list of file paths (relative to the project root) that the plugin
    /// needs write access to. Paths can be simple filenames like "Cargo.toml"
    /// or glob patterns like "packages/*/package.json".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_write_paths: Vec<String>,
    /// If true, the plugin declares that it may need to access the network.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub requires_network: bool,
}

/// A standardized error structure returned by the plugin on failure.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorResponse {
    /// A machine-readable error code.
    pub code: ErrorCode,
    /// A human-readable error message describing the failure.
    pub message: String,
}

/// A machine-readable classification of plugin errors.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// An error occurred during file I/O.
    IoError,
    /// Failed to parse a project file (e.g., invalid TOML).
    FileParseError,
    /// A required value was not found in a project file (e.g., missing 'version' key).
    ValueNotFoundError,
    /// The JSON request from the host was malformed.
    JsonRequestParseError,
    /// The plugin received an unknown or unsupported command.
    UnsupportedCommand,
    /// A generic failure for plugin-specific logic.
    PluginLogicFailed,
}
