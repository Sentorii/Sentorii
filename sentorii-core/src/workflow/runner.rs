use crate::workflow::state::{PersistentWorkflowState, delete_state, save_state};
use sentorii_contracts::command::CommandStep;
use sentorii_contracts::event::{Event, FailureInfo, StepStatus, StringInputRequest};
use sentorii_contracts::runner::CommandRunner;
use sentorii_contracts::step::Step;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Debug)]
pub struct Workflow<R: CommandRunner> {
    id: Uuid,
    event_tx: mpsc::Sender<Event>,
    runner: Arc<R>,
    state: PersistentWorkflowState,
    git_root: PathBuf,
}

impl<R: CommandRunner + Send + Sync + 'static> Workflow<R> {
    pub fn new(
        id: Uuid,
        event_tx: mpsc::Sender<Event>,
        runner: R,
        state: PersistentWorkflowState,
        git_root: PathBuf,
    ) -> Self {
        Self {
            id,
            event_tx,
            runner: Arc::new(runner),
            state,
            git_root,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let execution_result = async {
            let steps = self.state.steps.clone();

            for (index, step) in steps.iter().enumerate() {
                self.state.current_step = index;

                save_state(&self.git_root, &self.state)
                    .await
                    .context("Failed to save state before step execution")?;

                self.execute_step(index as u32, step)
                    .await
                    .map_err(|e| e.context(format!("Step {} failed", index)))?;
            }

            delete_state(&self.git_root)
                .await
                .context("Failed to delete state")?;
            self.event_tx.send(Event::WorkflowComplete(Ok(()))).await?;
            Ok(());
        }
        .await;

        let final_event = match execution_result {
            Ok(()) => Event::WorkflowComplete(Ok(())),
            Err(e) => {
                self.handle_failure(
                    &e,
                    self.state.current_step,
                    &self.state.steps[self.state.current_step],
                )
                .await;
                Event::WorkflowComplete(Err(e.to_string()))
            }
        };
        let _ = self.event_tx.send(final_event).await;
    }

    async fn execute_step(&mut self, index: u32, step: &Step) -> Result<()> {
        match step {
            Step::Command(cmd_step) => self.execute_command_step(index, cmd_step).await,
            Step::RequestStringInput(request_template) => {
                self.execute_string_input_step(request_template).await
            }
            Step::RequestSelectInput(request_template) => {
                unimplemented!("Select input is not yet supported by the runner.");
            }
        }
    }

    async fn execute_command_step(&self, index: u32, cmd_step: &CommandStep) -> Result<()> {
        self.event_tx.send(Event::StepStarted(index)).await?;
        let resolved_command = self.resolve_command_placeholders(cmd_step.command.clone())?;
        let result = self.runner.execute(resolved_command).await;

        let status = match &result {
            Ok(_) => StepStatus::Success,
            Err(e) => StepStatus::Failure(e.to_string()),
        };
        self.event_tx
            .send(Event::StepFinished(index, status))
            .await?;
        result
    }

    async fn execute_string_input_step(
        &mut self,
        request_template: &StringInputRequest,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        let request_to_send = StringInputRequest {
            key: request_template.key.clone(),
            prompt: request_template.prompt.clone(),
            tx,
        };

        self.event_tx
            .send(Event::StringInputRequired(request_to_send))
            .await?;

        let user_input = rx.await.context("UI failed to provide required input")?;
        self.state
            .context
            .insert(request_template.key.clone(), user_input);
        Ok(())
    }

    async fn handle_failure(&mut self, error: &anyhow::Error, index: usize, step: &Step) {
        self.state.status = "PausedOnFailure".to_string();
        let _ = save_state(&self.git_root, &self.state).await;

        if let Step::Command(cmd_step) = step {
            let failure_info = FailureInfo {
                error_message: error.to_string(),
                failed_command: cmd_step.clone(),
                possible_reverts: cmd_step.reverts.clone(),
                possible_recoveries: cmd_step.recoveries.clone(),
            };
            let _ = self
                .event_tx
                .send(Event::WorkflowPausedOnFailure(failure_info))
                .await;
        }
    }
}
