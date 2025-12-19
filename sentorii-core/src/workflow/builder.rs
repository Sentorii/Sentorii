//! A builder for constructing a `Workflow` object and emitting initial events.

use crate::error::CoreError;
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
pub struct Workflow<R: CommandRunner> {
    id: Uuid,
    event_tx: mpsc::Sender<Event>,
    runner: Arc<R>,
    state: PersistentWorkflowState,
    git_root: PathBuf,
}

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
        let static_steps = self.steps.iter().map(|step| step.static_info()).collect();
        let metadata = WorkflowMetadata::None;

        self.event_tx
            .send(Event::WorkflowPlanReady(static_steps, metadata))
            .await
            .map_err(|_| CoreError::EventChannelClosed)?;

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
mod tests {
    use super::*;
    use sentorii_contracts::step::Category;

    #[test]
    fn test_builder_accumulates_steps_correctly() {
        let (tx, _rx) = mpsc::channel(1);
        let builder = WorkflowBuilder::new(Uuid::new_v4(), tx, HashMap::new());

        let step1 = Step::Command(CommandStep {
            category: Category::Pull,
            command: SentoriiCommand::git_pull("origin", "develop"),
            display_text: "git pull".to_string(),
        });

        let final_builder = builder.step(step1.clone());
        assert_eq!(final_builder.steps.len(), 1);
        assert_eq!(final_builder.steps[0], step1);
    }

    #[tokio::test]
    async fn test_builder_emits_events_correctly_on_build() {
        let (event_tx, mut event_rx) = mpsc::channel(10);
        let workflow_id = Uuid::new_v4();
        let step1 = Step::Command(CommandStep {
            category: Category::Pull,
            command: SentoriiCommand::git_pull("origin", "develop"),
            display_text: "git pull".to_string(),
        });

        let steps = vec![step1.clone()];

        let workflow = WorkflowBuilder::new(workflow_id, event_tx, HashMap::new())
            .step(step1.clone())
            .build()
            .await
            .unwrap();

        let event1 = event_rx.recv().await.unwrap();
        assert!(matches!(event1, Event::WorkflowPlanReady(..)));
    }
}
