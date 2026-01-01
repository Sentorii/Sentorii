//! Workflow definitions for feature branches.

use crate::error::CoreError;
use crate::workflow::builder::WorkflowBuilder;
use crate::workflow::runner::Workflow;
use sentorii_contracts::context::{Context, ContextKey};
use sentorii_contracts::event::Event;
use sentorii_contracts::runner::CommandRunner;
use sentorii_contracts::step::{git_checkout, git_checkout_new_branch, git_pull, RequestStringInputTemplate};
use std::path::PathBuf;
use tokio::sync::mpsc;
use uuid::Uuid;
use sentorii_contracts::step::Step::RequestStringInput;

pub async fn start_feature<R: CommandRunner>(
    workflow_id: Uuid,
    event_tx: mpsc::Sender<Event>,
    runner: R,
    git_root: PathBuf,
    context: Context,
) -> Result<Workflow<R>, CoreError> {
    let name = "Feature Start".to_string();
    let workflow = WorkflowBuilder::new(workflow_id, name, event_tx, runner, git_root, context)
        .step(git_pull(ContextKey::Remote, ContextKey::Develop))
        .step(git_checkout(ContextKey::Develop))
        .step(RequestStringInput(RequestStringInputTemplate {
            key: "feature_name".to_string(),
            prompt: "Please provide the name for your new feature branch:".to_string(),
            default_value: None,
        }))
        .step(git_checkout_new_branch(ContextKey::FeatureBranch))
        .build()
        .await?;

    Ok(workflow)
}
