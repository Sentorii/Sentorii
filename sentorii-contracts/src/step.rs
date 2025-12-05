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
