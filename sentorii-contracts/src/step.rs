//! Defines different versions of steps.

use crate::command::CommandStep;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A high-level, UI-friendly category for a command step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Check,
    Checkout,
    Pull,
    Merge,
    Push,
    Tag,
    DeleteBranch,
    Plugin,
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
