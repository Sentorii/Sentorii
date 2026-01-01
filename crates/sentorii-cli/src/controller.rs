use std::time::Duration;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::oneshot;
use tui_input::backend::crossterm::EventHandler;
use sentorii_contracts::ui::{ModalState, UiState};
use crate::app::{ActiveModal, TuiAppState};

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

    if let ModalState::Failure { .. } = &state.canonical_state.modal {
        return Action::Quit;
    }

    if let Some(mut active_modal) = state.active_modal.take() {
        match &mut active_modal {
            ActiveModal::TextInput { widget, responder } => {
                match key.code {
                    KeyCode::Enter => {
                        if let Some(ActiveModal::TextInput { widget, responder }) = state.active_modal.take() {
                            state.canonical_state.modal = ModalState::None;
                            return Action::SubmitTextInput {
                                text: widget.value().to_string(),
                                responder
                            };
                        }
                    }
                    KeyCode::Esc => {
                        state.canonical_state.modal = ModalState::None;
                        return Action::Quit
                    }
                    _ => {
                        widget.handle_event(&event::Event::Key(key));
                    }
                }
            }
        }
        state.active_modal = Some(active_modal);
        return Action::NoOp;
    }

    Action::NoOp
}