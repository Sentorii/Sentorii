//! A builder for constructing a `Workflow` object and emitting initial events.

use crate::workflow::runner::Workflow;
use crate::workflow::state::PersistentWorkflowState;
use sentorii_contracts::command::CommandStep;
use sentorii_contracts::event::{Event, WorkflowMetadata};
use sentorii_contracts::step::Step;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct WorkflowBuilder {
    id: Uuid,
    event_tx: mpsc::Sender<Event>,
    steps: Vec<Step>,
    initial_context: HashMap<String, String>,
}

impl WorkflowBuilder {
    pub const fn new(
        workflow_id: Uuid,
        event_tx: mpsc::Sender<Event>,
        initial_context: HashMap<String, String>,
    ) -> Self {
        Self {
            id: workflow_id,
            event_tx,
            steps: Vec::new(),
            initial_context,
        }
    }

    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    pub async fn build(self) -> Result<Workflow> {
        self.event_tx
            .send(Event::WorkflowPlanReady(
                self.steps.clone(),
                WorkflowMetadata::None,
            ))
            .await?;

        let state = PersistentWorkflowState {
            workflow_id: self.id,
            current_step: 0,
            steps: self.steps.clone(),
            context: self.initial_context,
            status: "Running".to_string(),
        };

        Ok(Workflow {
            id: self.id,
            event_tx: self.event_tx,
            state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentorii_contracts::command::SentoriiCommand;
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
