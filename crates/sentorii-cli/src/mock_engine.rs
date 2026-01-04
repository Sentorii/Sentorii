#![allow(clippy::too_many_lines)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::missing_panics_doc)]

use log::{info, warn};
use sentorii_contracts::event::{
    Event, LogLine, RuntimeStepInfo, StringInputRequest, WorkflowMetadata,
};
use sentorii_contracts::ui::{UiStep, UiStepStatus};
use sentorii_contracts::workflow_request::WorkflowRequest;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

#[must_use]
pub fn start_mock_engine(
    mut request_rx: mpsc::Receiver<WorkflowRequest>,
    event_tx: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!("✅ Mock Core Engine has started.");

        // Wait for the initial request from the TUI.
        if let Some(WorkflowRequest::StartFeature { .. }) = request_rx.recv().await {
            info!("Mock Engine received FeatureStart request. Starting script...");

            // --- The Mock Script ---

            // 1. Send the initial workflow state.
            let steps = vec![
                UiStep {
                    id: 1,
                    description: "checkout main".into(),
                    status: UiStepStatus::Pending,
                    logs: vec![],
                },
                UiStep {
                    id: 2,
                    description: "pull".into(),
                    status: UiStepStatus::Pending,
                    logs: vec![],
                },
                UiStep {
                    id: 3,
                    description: "branch name".into(),
                    status: UiStepStatus::Pending,
                    logs: vec![],
                },
                UiStep {
                    id: 4,
                    description: "create branch".into(),
                    status: UiStepStatus::Pending,
                    logs: vec![],
                },
            ];
            event_tx
                .send(Event::WorkflowPlanReady(
                    "Mock Feature Start".into(),
                    steps,
                    WorkflowMetadata::None,
                ))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(500)).await;

            // 2. Simulate the first two steps succeeding.
            event_tx
                .send(Event::StepStarted(RuntimeStepInfo {
                    index: 1,
                    description: "checkout main".to_string(),
                    status: UiStepStatus::Running,
                }))
                .await
                .unwrap();
            event_tx
                .send(Event::LogOutput {
                    step_id: 1,
                    line: LogLine::Stderr("Trying again".to_string()),
                })
                .await
                .unwrap();
            event_tx
                .send(Event::LogOutput {
                    step_id: 1,
                    line: LogLine::Stdout("Checking out successful".to_string()),
                })
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(500)).await;
            event_tx
                .send(Event::StepFinished(RuntimeStepInfo {
                    index: 1,
                    description: "checkout main".to_string(),
                    status: UiStepStatus::Success,
                }))
                .await
                .unwrap();

            event_tx
                .send(Event::StepStarted(RuntimeStepInfo {
                    index: 2,
                    description: "pull".to_string(),
                    status: UiStepStatus::Running,
                }))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(800)).await;
            event_tx
                .send(Event::StepFinished(RuntimeStepInfo {
                    index: 2,
                    description: "pull".to_string(),
                    status: UiStepStatus::Success,
                }))
                .await
                .unwrap();

            // 3. Request user input.
            info!("Mock Engine requesting user input...");
            event_tx
                .send(Event::StepStarted(RuntimeStepInfo {
                    index: 3,
                    description: "branch name".to_string(),
                    status: UiStepStatus::Running,
                }))
                .await
                .unwrap();
            let (tx, rx) = oneshot::channel();
            event_tx
                .send(Event::StringInputRequired(StringInputRequest {
                    key: "feature_branch_name".into(),
                    prompt: "Enter feature branch name:".into(),
                    tx,
                }))
                .await
                .unwrap();

            // 4. Wait for the user's input from the TUI.
            if let Ok(branch_name) = rx.await {
                info!("Mock Engine received branch name: '{branch_name}'");
                event_tx
                    .send(Event::StepStarted(RuntimeStepInfo {
                        index: 3,
                        description: "branch name".to_string(),
                        status: UiStepStatus::Success,
                    }))
                    .await
                    .unwrap();

                // 5. Finish the workflow.
                tokio::time::sleep(Duration::from_millis(500)).await;
                event_tx
                    .send(Event::StepStarted(RuntimeStepInfo {
                        index: 4,
                        description: "create branch".to_string(),
                        status: UiStepStatus::Running,
                    }))
                    .await
                    .unwrap();
                tokio::time::sleep(Duration::from_millis(800)).await;
                event_tx
                    .send(Event::StepFinished(RuntimeStepInfo {
                        index: 4,
                        description: "create branch".to_string(),
                        status: UiStepStatus::Success,
                    }))
                    .await
                    .unwrap();
                event_tx
                    .send(Event::WorkflowComplete(Ok(())))
                    .await
                    .unwrap();
                info!("Mock workflow complete!");
            } else {
                warn!("Mock Engine: TUI closed the channel while awaiting input. Shutting down.");
                event_tx
                    .send(Event::StepFinished(RuntimeStepInfo {
                        index: 3,
                        description: "branch name".to_string(),
                        status: UiStepStatus::Failure("Cancelled by user.".into()),
                    }))
                    .await
                    .unwrap();
                event_tx
                    .send(Event::WorkflowComplete(Err("Cancelled by user.".into())))
                    .await
                    .unwrap();
            }
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        info!("🛑 Mock Core Engine shut down.");
    })
}
