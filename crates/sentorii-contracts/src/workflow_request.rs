//! Defines the commands that a UI can send to the core engine.

use crate::event::{RecoveryAction, RevertAction};
use serde::{Deserialize, Serialize};

/// Represents a message sent from a UI to the core engine to initiate or respond to actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WorkflowRequest {
    // --- Workflow Starters ---
    /// Request to start the "start feature" workflow.
    StartFeature { branch_name: Option<String> },
    /// Request to start the "finish feature" workflow.
    FinishFeature { branch_name: Option<String> },

    // --- User Responses ---
    /// A user submits text in response to an input prompt.
    SubmitTextInput(String),
    /// A user selects an option from a list.
    SubmitSelectInput(String),
    /// A user chooses a recovery action after a failure.
    ChooseRecoveryAction(RecoveryAction),
    /// A user chooses a revert action after a failure.
    ChooseRevertAction(RevertAction),
}
