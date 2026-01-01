//! The core dispatcher loop and state management for the engine.
//!
//! This module is responsible for listening for incoming `WorkflowRequest` messages
//! and ensuring that only one workflow is executed at a time.

use crate::error::{CoreError, InvalidStateError};
use crate::git::executor::CommandExecutor;
use crate::git::utils::resolve_git_root;
use crate::workflow::context::ContextProvider;
use crate::workflow::definition::feature::start_feature;
use log::{error, info, warn};
use sentorii_config::load_config;
use sentorii_contracts::event::Event;
use sentorii_contracts::workflow_request::WorkflowRequest;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Represents the live, in-memory state of the engine.
enum EngineState {
    Idle,
    Busy { workflow_id: Uuid },
}

#[derive(Debug)]
struct DispatcherPoisonedError(String);

impl Display for DispatcherPoisonedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dispatcher state mutex was poisoned: {}", self.0)
    }
}

/// Spawns the background Tokio task that runs the main "dispatcher loop".
#[must_use]
pub fn start_engine(
    mut request_rx: mpsc::Receiver<WorkflowRequest>,
    event_tx: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    let engine_state = Arc::new(Mutex::new(EngineState::Idle));

    tokio::spawn(async move {
        info!("Sentorii Core Engine started");

        while let Some(request) = request_rx.recv().await {
            info!("Sentorii Core Engine received request: {:?}", request);
            if let Err(poisoned_error) = handle_request(request, &engine_state, event_tx.clone()) {
                error!(
                    "FATAL: Engine state mutex is poisoned, shutting down dispatcher. Error: {poisoned_error}"
                );
                break;
            }
        }

        info!("Sentorii Core Core Engine shut down");
    })
}

fn handle_request(
    request: WorkflowRequest,
    engine_state: &Arc<Mutex<EngineState>>,
    event_tx: mpsc::Sender<Event>,
) -> Result<(), DispatcherPoisonedError> {
    let mut state_guard = engine_state
        .lock()
        .map_err(|e| DispatcherPoisonedError(e.to_string()))?;

    match *state_guard {
        EngineState::Idle => {
            let workflow_id = Uuid::new_v4();
            *state_guard = EngineState::Busy { workflow_id };
            drop(state_guard);
            let engine_state_clone = Arc::clone(engine_state);
            spawn_workflow_execution_task(request, workflow_id, engine_state_clone, event_tx);
        }
        EngineState::Busy { workflow_id } => {
            warn!("Rejected request; engine is busy with workflow {workflow_id}");
            let reason = format!("Engine is busy with an existing workflow: {workflow_id}");

            tokio::spawn(async move {
                let _ = event_tx.send(Event::WorkflowRejected { reason }).await;
            });
        }
    }
    Ok(())
}

fn spawn_workflow_execution_task(
    request: WorkflowRequest,
    workflow_id: Uuid,
    engine_state: Arc<Mutex<EngineState>>,
    event_tx: mpsc::Sender<Event>,
) {
    info!("Engine is now busy with new workflow {workflow_id}");

    tokio::spawn(async move {
        let _result = dispatch_workflow(request, event_tx, workflow_id).await;

        match engine_state.lock() {
            Ok(mut state) => {
                if let EngineState::Busy {
                    workflow_id: active_id,
                } = *state
                    && active_id == workflow_id
                {
                    *state = EngineState::Idle;
                    info!("Workflow {workflow_id} finished. Engine is now idle");
                }
            }
            Err(e) => {
                error!("FATAL: Mutex poisoned while resetting engine state to idle. Error: {e}");
            }
        }
    });
}

async fn dispatch_workflow(
    request: WorkflowRequest,
    event_tx: mpsc::Sender<Event>,
    workflow_id: Uuid,
) -> Result<(), CoreError> {
    let git_root = resolve_git_root().await?;
    let runner = CommandExecutor::new(event_tx.clone());

    let config = load_config()?;
    let context = config.to_context();

    let workflow = match request {
        WorkflowRequest::StartFeature { .. } => {
            start_feature(workflow_id, event_tx, runner, git_root, context).await?
        }
        _ => {
            return Err(CoreError::InvalidState(InvalidStateError::NotRunnable(
                "Workflow request type is not yet implemented.".to_string(),
            )));
        }
    };

    workflow.run().await;

    Ok(())
}
