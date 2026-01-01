use tokio::sync::mpsc::Sender;
use sentorii_contracts::ui::{ModalState, UiState};
use sentorii_contracts::workflow_request::WorkflowRequest;
use anyhow::Result;
use tokio::sync::oneshot;
use tui_input::Input;
use sentorii_contracts::event::{Event, StringInputRequest};
use crate::controller::Action;
use crate::state;

pub enum ActiveModal {
    TextInput {
        widget: Input,
        responder: oneshot::Sender<String>,
    }
}

pub struct TuiAppState {
    pub canonical_state: UiState,
    pub active_modal: Option<ActiveModal>,
}

pub struct App {
    pub tui_state: TuiAppState,
    should_quit: bool,
    request_tx: Sender<WorkflowRequest>,
}

impl App {
    pub fn new(request_tx: Sender<WorkflowRequest>) -> Self {
        Self {
            tui_state: TuiAppState {
                canonical_state: UiState::default(),
                active_modal: None,
            },
            should_quit: false,
            request_tx,
        }
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::SubmitTextInput { text, responder } => {
                let _ = responder.send(text);
            }
            Action::Quit => self.should_quit = true,
            Action::NoOp => {}
        }
        Ok(())
    }

    pub fn handle_core_event(&mut self, event: Event) {
       match event {
           Event::StringInputRequired(StringInputRequest { key, prompt, tx}) => {
               self.tui_state.canonical_state.modal = ModalState::TextInput {
                   key,
                   prompt,
                   buffer: String::new(),
               };

               self.tui_state.active_modal = Some(ActiveModal::TextInput {
                   widget: Input::default(),
                   responder: tx,
               })
           }
           _ => {
               state::update_state(&mut self.tui_state.canonical_state, event);

               if !matches!(self.tui_state.canonical_state.modal, ModalState::TextInput {..}) {
                   self.tui_state.active_modal = None;
               }
           }
       }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

