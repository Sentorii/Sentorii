use crate::error::InvalidStateError::{EventChannelClosed, InputChannelClosed};
use crate::error::{CoreError, InvalidStateError};
use crate::workflow::state::{PersistentWorkflowState, delete_state, save_state};
use log::{Level, log};
use sentorii_contracts::command::Command;
use sentorii_contracts::event::{
    Event, FailureInfo, RuntimeStepInfo, StepStatus, StringInputRequest,
};
use sentorii_contracts::runner::CommandRunner;
use sentorii_contracts::step::{CommandStep, RequestStringInputTemplate, Step};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Debug)]
pub struct Workflow<R: CommandRunner> {
    pub id: Uuid,
    pub event_tx: mpsc::Sender<Event>,
    pub runner: Arc<R>,
    pub state: PersistentWorkflowState,
    pub git_root: PathBuf,
}

impl<R: CommandRunner + Send + Sync + 'static> Workflow<R> {
    pub async fn run(mut self) {
        log!(Level::Info, "Starting workflow {:?}", self.id);
        let result = self.run_internal().await;

        let final_event = match result {
            Ok(()) => Event::WorkflowComplete(Ok(())),
            Err(e) => Event::WorkflowComplete(Err(e.to_string())),
        };
        let _ = self.event_tx.send(final_event).await;
    }

    async fn run_internal(&mut self) -> Result<(), CoreError> {
        let steps = self.state.steps.clone();
        for (index, step) in steps.iter().enumerate() {
            self.state.current_step = index;

            save_state(&self.git_root, &self.state).await?;

            let index_u32 = u32::try_from(index).map_err(|_| {
                CoreError::InvalidState(InvalidStateError::NotRunnable(format!(
                    "Index conversion failed for: {index}"
                )))
            });

            self.execute_step(index_u32?, step).await?;
        }

        delete_state(&self.git_root).await?;
        Ok(())
    }

    async fn execute_step(&mut self, index: u32, step: &Step) -> Result<(), CoreError> {
        let resolved_description = step.resolved_description(&self.state.context);
        self.event_tx
            .send(Event::StepStarted(RuntimeStepInfo {
                index,
                description: resolved_description,
            }))
            .await
            .map_err(|_| EventChannelClosed)?;

        let result = match step {
            Step::Command(cmd_step) => self.execute_command_step(cmd_step).await,
            Step::RequestStringInput(request_template) => {
                self.execute_string_input_step(request_template).await
            }
            Step::RequestSelectInput(_) => {
                unimplemented!("Select input is not yet supported by the runner.");
            }
        };

        match result {
            Ok(()) => {
                self.event_tx
                    .send(Event::StepFinished(index, StepStatus::Success))
                    .await
                    .map_err(|_| EventChannelClosed)?;
                Ok(())
            }
            Err(e) => {
                self.handle_failure(&e, index, step).await;
                Err(e)
            }
        }
    }

    async fn execute_command_step(&self, command_step: &CommandStep) -> Result<(), CoreError> {
        let executable = command_step.to_executable(&self.state.context)?;
        self.runner.execute(executable).await?;
        Ok(())
    }

    async fn execute_string_input_step(
        &mut self,
        request_template: &RequestStringInputTemplate,
    ) -> Result<(), CoreError> {
        let (tx, rx) = oneshot::channel();
        let request_to_send = StringInputRequest {
            key: request_template.key.clone(),
            prompt: request_template.prompt.clone(),
            tx,
        };

        self.event_tx
            .send(Event::StringInputRequired(request_to_send))
            .await
            .map_err(|_| EventChannelClosed)?;

        let user_input = rx.await.map_err(|_| InputChannelClosed)?;

        match request_template.key.as_str() {
            "tag" => {
                self.state.context.set_tag(user_input);
            }
            "feauture_name" => {
                self.state.context.set_feature_branch(user_input);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_failure(&mut self, error: &CoreError, index: u32, step: &Step) {
        self.state.status = "PausedOnFailure".to_string();
        let _ = save_state(&self.git_root, &self.state).await;

        if let Step::Command(cmd_step) = step {
            let failure_info = FailureInfo {
                error_message: error.to_string(),
                failed_command: cmd_step.clone(),
                possible_reverts: cmd_step.possible_reverts(),
                possible_recoveries: cmd_step.possible_recoveries(),
            };
            let _ = self
                .event_tx
                .send(Event::WorkflowPausedOnFailure(failure_info))
                .await;
        }

        let _ = self
            .event_tx
            .send(Event::StepFinished(
                index,
                StepStatus::Failure(error.to_string()),
            ))
            .await;
    }
}
