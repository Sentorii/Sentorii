//! Defines the shared, UI-agnostic data structures for UI state.

use crate::event::FailureInfo;
use serde::{Deserialize, Serialize};

/// The status of a single step as viewed by the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "error_message")]
pub enum UiStepStatus {
    Pending,
    Running,
    Success,
    Failure(String),
}

/// A single step in the UI's view of the workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStep {
    /// The unique identifier for this step, matching the one from the engine's plan.
    pub id: usize,
    /// The underlying command details for display purposes.
    pub description: String,
    /// The current status of this step.
    pub status: UiStepStatus,
    /// A collection of logs captured during this step's execution.
    pub logs: Vec<String>,
}

/// Represents the state of a modal dialog or blocking interaction in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ModalState {
    None,
    TextInput {
        /// The message to display to the user.
        prompt: String,
        /// The unique key identifying this input request.
        key: String,
        /// The current text buffer, managed by the UI.
        buffer: String,
    },
    SelectInput {
        /// The message to display to the user.
        prompt: String,
        /// The unique key identifying this input request.
        key: String,
        /// The list of options to display.
        options: Vec<String>,
        /// The index of the currently selected option, managed by the UI.
        selected_index: usize,
    },
    Failure {
        /// The detailed failure information from the engine.
        info: FailureInfo,
        /// The index of the currently selected recovery/revert action, managed by the UI.
        selected_action_index: Option<usize>,
    },
}

/// The main state modal for any Sentorii user interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiState {
    /// The title of the currently running workflow.
    pub workflow_title: String,
    /// The list of all steps in the current workflow.
    pub steps: Vec<UiStep>,
    /// The current modal state of the UI. If not `None`, the UI should display a modal.
    pub modal: ModalState,
}
