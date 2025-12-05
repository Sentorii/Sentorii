use crate::command::SentoriiCommand;
use serde::{Deserialize, Serialize};

/// A high-level, UI-friendly category for a command step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepCategory {
    Check,
    Checkout,
    Pull,
    Merge,
    Push,
    Tag,
    DeleteBranch,
    Plugin,
}

/// A rich object representing a command to be executed as part of a workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandStep {
    /// The high-level category of the command.
    pub category: StepCategory,
    /// The specific, executable command.
    pub command: SentoriiCommand,
    /// The human-readable string to be displayed in the UI.
    pub display_text: String,
}

/// The declarative representation of a single step within a workflow plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    /// A step that executes a command.
    Command(CommandStep),
    /// A step that pauses the workflow to request a string input from the user.
    RequestStringInput {
        /// A unique key to identify this input request.
        key: String,
        /// The message to display to the user.
        prompt: String,
        /// An optional default value for the input.
        default_value: Option<String>,
    },
    /// A step that pauses the workflow to request a selection from the user.
    RequestSelectInput {
        /// A unique key to identify this input request.
        key: String,
        /// The message to display to the user.
        prompt: String,
        /// A list of options to display to the user.
        options: Vec<String>,
        /// An optional default option to pre-select.
        default_option: Option<String>,
    },
}
