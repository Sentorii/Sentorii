#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use sentorii_cli::cli::Cli;
use sentorii_cli::tui::Tui;
use sentorii_cli::{App, controller, ui, workflow_dispatcher};
use sentorii_contracts::event::Event;
use sentorii_contracts::workflow_request::WorkflowRequest;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (request_tx, request_rx) = mpsc::channel::<WorkflowRequest>(10);
    let (event_tx, mut event_rx) = mpsc::channel::<Event>(1000);

    tokio::spawn(sentorii_core::start_engine(request_rx, event_tx));

    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(request_tx.clone());
    workflow_dispatcher::dispatch(cli, &request_tx)?;

    let tick_rate = Duration::from_millis(16);
    while !app.should_quit() {
        tui.draw(|frame| ui::render(frame, &mut app.tui_state))?;

        if let Some(action) = controller::poll_for_action(tick_rate, &mut app.tui_state)? {
            app.handle_action(action)?;
        }

        if let Ok(core_event) = event_rx.try_recv() {
            app.handle_core_event(core_event);
        }
    }

    Ok(())
}
