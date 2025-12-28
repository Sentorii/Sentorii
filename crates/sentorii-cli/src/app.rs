use tokio::sync::mpsc::Sender;
use sentorii_contracts::ui::UiState;
use sentorii_contracts::workflow_request::WorkflowRequest;
use anyhow::Result;
use tui_input::Input;
use sentorii_contracts::event::Event;
use crate::controller::Action;
use crate::state;

pub struct App {
    pub state: UiState,
    should_quit: bool,
    request_tx: Sender<WorkflowRequest>,
}

impl App {
    pub fn new(request_tx: Sender<WorkflowRequest>) -> Self {
        Self {
            state: UiState::default(),
            should_quit: false,
            request_tx,
        }
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::SendRequest(req) => self.request_tx.send(req),
            Action::Quit => self.should_quit = true,
            Action::NoOp => {}
        }
        Ok(())
    }

    pub fn handle_core_event(&mut self, event: Event) {
        state::update_state(&mut self.state, event);
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit || self.state.is_finished()
    }
}

pub struct TuiAppState {
    pub canonical_state: UiState,
    pub input_widget: Option<Input>,
}