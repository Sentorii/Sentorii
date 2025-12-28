use sentorii_contracts::event::Event;
use sentorii_contracts::ui::{ModalState, UiState};

pub fn update_state(state: &mut UiState, event: Event) {
    match event {
        Event::WorkflowPlanReady(name, steps, workflowMetaData) => {
            state.workflow_title = name;
            state.steps = steps;
            state.modal = ModalState::None;
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
        Event::StringInputRequired(prompt) => {
            state.modal = ModalState::TextInput {
                prompt: prompt.prompt,
                key: prompt.key,
                buffer: String::new(),
            }
        }
        Event::WorkflowPausedOnFailure(failure_info) => {
            state.modal = ModalState::Failure {
                info: failure_info,
                selected_action_index: None,
            }
        }
    }
}