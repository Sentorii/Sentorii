//! Manages the persistent stage of a running or failed workflow.
//!
//! This module is responsible for atomically saving, loading, and deleting the
//! serializable state of a workflow to the filesystem within the `.sentorii`
//! directory of the current Git repository.

use crate::error::CoreError;
use sentorii_contracts::context::Context;
use sentorii_contracts::step::Step;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, read_to_string, remove_file, rename, write};
use uuid::Uuid;

const STATE_FILENAME: &str = "workflow-state.json";

/// Represents the serializable state of a potentially paused workflow.
/// This struct is the "memory" of a workflow, allowing it to be resumed or reverted.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PersistentWorkflowState {
    pub workflow_id: Uuid,
    /// The index of the step that is about to be executed or has just failed.
    pub current_step: usize,
    /// A complete copy of all steps in the workflow plan.
    pub steps: Vec<Step>,
    /// A map of key-value pairs, collected from user input or initial parameters.
    pub context: Context,
    /// The current condition of the workflow, e.g., "Running", or "`PausedOnFailure`".
    pub status: String,
}

/// Returns the path for the `.sentorii` directory within the repo.
fn get_sentorii_dir(git_root: &Path) -> PathBuf {
    git_root.join(".sentorii/")
}

/// Returns the conventional file path for a workflow's state file.
fn get_state_path(git_root: &Path) -> PathBuf {
    get_sentorii_dir(git_root).join(STATE_FILENAME)
}

/// Asynchronously saves the workflow's state to a file using an atomic write-and-rename operation.
pub async fn save_state(git_root: &Path, state: &PersistentWorkflowState) -> Result<(), CoreError> {
    let sentorii_dir = get_sentorii_dir(git_root);
    create_dir_all(&sentorii_dir).await?;

    let final_path = get_state_path(git_root);
    let temp_path = final_path.with_extension("json.tmp");

    let state_json = serde_json::to_string_pretty(state)?;

    write(&temp_path, state_json).await?;
    rename(&temp_path, &final_path).await?;

    Ok(())
}

/// Asynchronously loads a workflow's state from a file.
pub async fn load_state(git_root: &Path) -> Result<PersistentWorkflowState, CoreError> {
    let path = get_state_path(git_root);
    let state_json = read_to_string(path).await?;
    let state = serde_json::from_str(&state_json)?;
    Ok(state)
}

/// Asynchronously deletes a workflow's state file.
pub async fn delete_state(git_root: &Path) -> Result<(), CoreError> {
    let path = get_state_path(git_root);
    if path.exists() {
        remove_file(path).await?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_get_state_path_constructs_correct_static_path() {
        let git_root = PathBuf::from("/tmp/project");
        let expected_state_path = PathBuf::from("/tmp/project/.sentorii/workflow-state.json");
        assert_eq!(get_state_path(&git_root), expected_state_path);
    }
}

#[cfg(test)]
#[cfg(feature = "test-integration")]
#[allow(clippy::unwrap_used)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    fn create_dummy_state() -> PersistentWorkflowState {
        PersistentWorkflowState {
            workflow_id: Uuid::new_v4(),
            current_step: 1,
            steps: vec![],
            context: HashMap::from([("key".to_string(), "value".to_string())]),
            status: "Running".to_string(),
        }
    }

    #[tokio::test]
    async fn test_save_and_load_state_happy_path() {
        let temp_dir = TempDir::new().unwrap();
        let git_root = temp_dir.path();
        let original_state = create_dummy_state();

        save_state(git_root, &original_state).await.unwrap();
        let loaded_state = load_state(git_root).await.unwrap();

        pretty_assertions::assert_eq!(original_state, loaded_state);
    }

    #[tokio::test]
    async fn test_delete_state() {
        let temp_dir = TempDir::new().unwrap();
        let git_root = temp_dir.path();
        let file_path = get_state_path(git_root);

        save_state(git_root, &create_dummy_state()).await.unwrap();
        assert!(file_path.exists());

        delete_state(git_root).await.unwrap();
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_load_state_file_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let git_root = temp_dir.path();
        let load_result = load_state(git_root).await;
        assert!(matches!(load_result, Err(CoreError::Io(_))));
    }
}
