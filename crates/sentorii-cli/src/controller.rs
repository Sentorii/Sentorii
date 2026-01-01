use crate::app::{ActiveModal, TuiAppState};
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

    if let ModalState::Failure { .. } = &state.canonical_state.modal {
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

    Action::NoOp
}
