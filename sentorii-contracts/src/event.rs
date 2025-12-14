//! Defines the complete, structured language for the backend to communicate its state.

use crate::command::CommandStep;
use crate::step::Step;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// The execution status of a single workflow step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "message")]
pub enum StepStatus {
    Success,
    Failure(String),
}

/// An action that reverts a failed step, typically returning to a clean git state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevertAction {
    SimpleCheckout { branch: String },
    MergeAbort,
}

/// An action that attempts to recover from a failed step and continue the workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    RetryStep,
    PullBeforeMerge,
}

/// A rich object containing all information about a workflow failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureInfo {
    pub error_message: String,
    pub failed_command: CommandStep,
    pub possible_reverts: Vec<RevertAction>,
    pub possible_recoveries: Vec<RevertAction>,
}

/// A request from the engine to the UI for a string input.
/// This struct cannot be serialized as it contains a channel sender.
#[derive(Debug)]
pub struct StringInputRequest {
    pub key: String,
    pub prompt: String,
    pub tx: oneshot::Sender<String>,
}

/// A request from the egnine to the UI for a selection from a list.
/// This struct cannot be serialized as it contains a channel sender.
#[derive(Debug)]
pub struct SelectInputRequest {
    pub key: String,
    pub prompt: String,
    pub options: Vec<String>,
    pub tx: oneshot::Sender<String>,
}

/// Represents a single Git commit with its essential information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub subject: String,
}

/// Contains metadata about the workflow being executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowMetadata {
    None,
    MergePreview { commits_to_merge: Vec<CommitInfo> },
}

/// A single step in a workflow plan identified by a unique ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifiedStep {
    pub id: u32,
    pub step: Step,
}

/// The primary enum representing all possible state changes containing the full execution plan.
#[derive(Debug)]
pub enum Event {
    /// Sent once at the beginning of a workflow, containing the full execution plan.
    WorkflowPlanReady(Vec<Step>, WorkflowMetadata),
    /// Sent when a specific step is about to be executed.
    StepStarted(u32),
    /// Sent when a specific step has finished, with its status.
    StepFinished(u32, StepStatus),
    /// Provides real-time log output from a running command.
    LogOutput(String),
    /// Sent once when the entire workflow has completed.
    WorkflowComplete(Result<(), String>),
    /// Sent when a step fails and the workflow is paused, awaiting user intervention.
    WorkflowPausedOnFailure(FailureInfo),
    /// Sent once when engine is busy, but a new workflow request is received.
    WorkflowRejected { reason: String },
    /// Sent when the engine requires a string input from the user to proceed.
    StringInputRequired(StringInputRequest),
    /// Sent when the engine requires a selection from the user to proceed.
    SelectInputRequired(SelectInputRequest),
}
