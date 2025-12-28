use std::time::Duration;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sentorii_contracts::ui::UiState;
use sentorii_contracts::workflow_request::WorkflowRequest;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    SendRequest(WorkflowRequest),
    Quit,
    NoOp,
}

pub fn poll_for_action(tick_rate: Duration, state: &mut UiState) -> Result<Option<Action>> {
    if event::poll(tick_rate)? {
        if let event::Event::Key(key) = event::read()? {
            return Ok(Some(handle_key_event(key, state)));
        }
    }

    Ok(None)
}

fn handle_key_event(key: KeyEvent, state: &mut UiState) -> Action {
    if let KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
        ..
    } = key
    {
        return Action::Quit;
    }

    if state.is_failed() {
        return Action::Quit;
    }

    if state.is_awaiting_input() {
        if let Some(input_state) = state.input_mut() {
            return match key.code {
                KeyCode::Enter => {
                    let user_input = input_state.value().to_string();
                    state.set_is_awaiting_input(false);
                    Action::SendRequest(WorkflowRequest::SubmitTextInput(user_input))
                }
                KeyCode::Esc => Action::Quit,
                _ => {
                    input_state.handle_event(&event::Event::Key(key));
                    Action::NoOp
                }
            };
        }
    }

    Action::NoOp
}