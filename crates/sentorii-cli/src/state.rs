use crate::App;
use sentorii_contracts::event::Event;
use sentorii_contracts::ui::{ModalState, UiStepStatus};

pub fn update_state(app: &mut App, event: Event) {
    let state = &mut app.tui_state.canonical_state;
    match event {
        Event::WorkflowPlanReady(name, steps, ..) => {
            state.workflow_title = name;
            state.steps = steps;
            state.modal = ModalState::None;
            state.status = UiStepStatus::Running;
        }
        Event::StepStarted(info) => {
            if let Some(step) = state.steps.iter_mut().find(|s| s.id == info.index) {
                step.status = info.status;
            }
        }
        Event::StepFinished(info) => {
            if let Some(step) = state.steps.iter_mut().find(|s| s.id == info.index) {
                step.status = info.status;
            }
        }
        Event::LogOutput { step_id, line } => {
            if let Some(step) = state.steps.iter_mut().find(|s| s.id == step_id) {
                step.logs.push(line);
            }
        },
        Event::StringInputRequired(prompt) => {
            state.modal = ModalState::TextInput {
                prompt: prompt.prompt,
                key: prompt.key,
                buffer: String::new(),
            }
        }
        Event::SelectInputRequired(prompt) => {
            state.modal = ModalState::SelectInput {
                prompt: prompt.prompt,
                key: prompt.key,
                options: prompt.options,
                selected_index: 0,
            }
        }
        Event::WorkflowPausedOnFailure(failure_info) => {
            state.modal = ModalState::Failure {
                info: failure_info.clone(),
                selected_action_index: None,
            };
            state.status = UiStepStatus::Failure(failure_info.error_message);
        }
        Event::WorkflowComplete(info) => match info {
            Ok(()) => {
                state.status = UiStepStatus::Success;
                app.should_quit = false;
            },
            Err(e) => {
                state.status = UiStepStatus::Failure(e);
                app.should_quit = false;
            }
        },
        Event::WorkflowRejected { reason } => {
            state.status = UiStepStatus::Failure(reason);
        }
    }
}
