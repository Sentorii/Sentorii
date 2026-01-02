use crate::app::{ActiveModal, FocusTarget, TuiAppState, ViewMode};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sentorii_contracts::ui::ModalState;
use std::time::Duration;
use tokio::sync::oneshot;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub enum Action {
    SubmitTextInput {
        text: String,
        responder: oneshot::Sender<String>,
    },
    Quit,
    NoOp,
}

pub fn poll_for_action(tick_rate: Duration, state: &mut TuiAppState) -> Result<Option<Action>> {
    if event::poll(tick_rate)? {
        if let event::Event::Key(key) = event::read()? {
            return Ok(Some(handle_key_event(key, state)));
        }
    }

    Ok(None)
}

fn handle_key_event(key: KeyEvent, state: &mut TuiAppState) -> Action {
    if let KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        ..
    } = key
    {
        return Action::Quit;
    }

    if let Some(active_modal) = state.active_modal.take() {
        return match active_modal {
            ActiveModal::TextInput {
                mut widget,
                responder,
            } => match key.code {
                KeyCode::Enter => {
                    state.canonical_state.modal = ModalState::None;
                    Action::SubmitTextInput {
                        text: widget.value().to_string(),
                        responder,
                    }
                }
                KeyCode::Esc => {
                    state.canonical_state.modal = ModalState::None;
                    Action::Quit
                }
                _ => {
                    widget.handle_event(&event::Event::Key(key));
                    state.active_modal = Some(ActiveModal::TextInput { widget, responder });
                    Action::NoOp
                }
            },
        };
    }

    if state.view_mode == ViewMode::StepDetail {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            state.view_mode = ViewMode::Normal;
            state.selected_step_id = None;
            return Action::NoOp;
        }
    }

    if key.code == KeyCode::Tab {
        state.focus = match state.focus {
            FocusTarget::Steps => {
                FocusTarget::Logs
            },
            FocusTarget::Logs => FocusTarget::Steps,
        };
        return Action::NoOp;
    }

    match state.focus {
        FocusTarget::Steps => handle_steps_input(key, state),
        FocusTarget::Logs => Action::NoOp
    }
}

fn handle_steps_input(key: KeyEvent, state: &mut TuiAppState) -> Action {
    let steps_count = state.canonical_state.steps.len();
    if steps_count == 0 {
        return Action::NoOp;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            let i = state.list_state.selected().map_or(0, |i| i.saturating_sub(1));
            state.list_state.select(Some(i));
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let i = state.list_state.selected().map_or(0, |i| (i + 1).min(steps_count - 1));
            state.list_state.select(Some(i));
        }
        KeyCode::Enter => {
            if let Some(selected_index) = state.list_state.selected() {
                if let Some(step) = state.canonical_state.steps.get(selected_index) {
                    state.selected_step_id = Some(step.id);
                    state.view_mode = ViewMode::StepDetail;
                }
            }
        }
        _ => {}
    }
    Action::NoOp
}
