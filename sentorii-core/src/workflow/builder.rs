//! A builder for constructing a `Workflow` object and emitting initial events.

use crate::error::CoreError;
use crate::error::InvalidStateError::EventChannelClosed;
use crate::workflow::runner::Workflow;
use crate::workflow::state::PersistentWorkflowState;
use sentorii_contracts::context::Context;
use sentorii_contracts::event::{Event, WorkflowMetadata};
use sentorii_contracts::runner::CommandRunner;
use sentorii_contracts::step::Step;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct WorkflowBuilder<R: CommandRunner> {
    id: Uuid,
    event_tx: mpsc::Sender<Event>,
    runner: R,
    git_root: PathBuf,
    steps: Vec<Step>,
    context: Context,
}

impl<R: CommandRunner> WorkflowBuilder<R> {
    pub const fn new(
        workflow_id: Uuid,
        event_tx: mpsc::Sender<Event>,
        runner: R,
        git_root: PathBuf,
        context: Context,
    ) -> Self {
        Self {
            id: workflow_id,
            event_tx,
            runner,
            git_root,
            steps: Vec::new(),
            context,
        }
    }

    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    pub async fn build(self) -> Result<Workflow<R>, CoreError> {
        let static_steps = self.steps.iter().map(Step::static_info).collect();
        let metadata = WorkflowMetadata::None;

        self.event_tx
            .send(Event::WorkflowPlanReady(static_steps, metadata))
            .await
            .map_err(|_| EventChannelClosed)?;

        let state = PersistentWorkflowState {
            workflow_id: self.id,
            current_step: 0,
            steps: self.steps,
            context: self.context,
            status: "Running".to_string(),
        };

        Ok(Workflow {
            id: self.id,
            event_tx: self.event_tx,
            runner: Arc::new(self.runner),
            state,
            git_root: self.git_root,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use sentorii_contracts::context::ContextBuilder;
    use sentorii_contracts::runner::MockCommandRunner;
    use sentorii_contracts::step::git_pull;

    #[test]
    fn test_builder_accumulates_steps_correctly() {
        let (tx, _rx) = mpsc::channel(1);
        let workflow_id = Uuid::new_v4();
        let runner = MockCommandRunner::default();
        let git_root = PathBuf::from("git_root");
        let context = ContextBuilder::new().build();
        let builder = WorkflowBuilder::new(workflow_id, tx, runner, git_root, context);

        let step1 = git_pull("origin", "new-branch");

        let final_builder = builder.step(step1.clone());
        assert_eq!(final_builder.steps.len(), 1);
        assert_eq!(final_builder.steps[0], step1);
    }

    #[tokio::test]
    async fn test_builder_emits_events_correctly_on_build() {
        let (event_tx, mut event_rx) = mpsc::channel(10);
        let workflow_id = Uuid::new_v4();
        let runner = MockCommandRunner::default();
        let git_root = PathBuf::from("git_root");
        let context = ContextBuilder::new().build();

        let step1 = git_pull("origin", "new-branch");

        let steps = vec![step1.clone()];

        let workflow = WorkflowBuilder::new(workflow_id, event_tx, runner, git_root, context)
            .step(step1.clone())
            .build()
            .await
            .unwrap();

        let event1 = event_rx.recv().await.unwrap();
        assert!(matches!(event1, Event::WorkflowPlanReady(..)));
        assert_eq!(steps, workflow.state.steps);
    }
}
