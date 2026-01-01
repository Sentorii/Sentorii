use crate::app::{ActiveModal, TuiAppState};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use tui_logger::TuiLoggerWidget;
use sentorii_contracts::command::Command;
use sentorii_contracts::ui::{ModalState, UiStepStatus};

pub fn render(frame: &mut Frame, app_state: &mut TuiAppState) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ].as_ref())
        .split(frame.area());
    let app_chunk = main_chunks[0];
    let log_chunk = main_chunks[1];
    let steps_list: Vec<ListItem> = app_state
        .canonical_state
        .steps
        .iter()
        .map(|step| {
            let (prefix, style) = match &step.status {
                UiStepStatus::Pending => ("  [ ]", Style::default().fg(Color::DarkGray)),
                UiStepStatus::Running => ("  [*]", Style::default().fg(Color::Yellow)),
                UiStepStatus::Success => ("  [✔]", Style::default().fg(Color::Green)),
                UiStepStatus::Failure(_) => ("  [✘]", Style::default().fg(Color::Red)),
            };
            let content = format!("{prefix} {}", step.description.clone());
            ListItem::new(Line::from(content).style(style))
        })
        .collect();

    let list_widget = List::new(steps_list).block(
        Block::default()
            .title(format!(" {} ", &app_state.canonical_state.workflow_title))
            .borders(Borders::ALL),
    );

    frame.render_widget(list_widget, app_chunk);

    match &app_state.canonical_state.modal {
        ModalState::TextInput { prompt, .. } => {
            if let Some(ActiveModal::TextInput { widget, .. }) = &app_state.active_modal {
                render_text_input_modal(frame, prompt, widget);
            }
        }
        ModalState::Failure { info, .. } => render_failure_modal(
            frame,
            &info.failed_command.static_description(),
            &info.error_message,
        ),
        _ => {}
    }

    let logger_widget = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(" Logs ")
                .borders(Borders::ALL),
        )
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Cyan))
        .style_debug(Style::default().fg(Color::Green))
        .style_trace(Style::default().fg(Color::Magenta));

    frame.render_widget(logger_widget, log_chunk)
}

fn render_text_input_modal(frame: &mut Frame, prompt: &str, input: &tui_input::Input) {
    let area = centered_rect(60, 3, frame.area());

    let block = Block::default()
        .title(prompt)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_paragraph = Paragraph::new(input.value()).block(block);

    frame.render_widget(Clear, area);

    frame.render_widget(input_paragraph, area);

    frame.set_cursor_position(Position::new(
        area.x + u16::try_from(input.visual_cursor()).unwrap() + 1,
        area.y + 1,
    ));
}

fn render_failure_modal(frame: &mut Frame, title: &str, error_message: &str) {
    let area = centered_rect(80, 25, frame.area());
    let block = Block::default()
        .title(format!(" Workflow Failed: {title}"))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let error_text = format!("{error_message}\n\nPress any key to exit.");

    let error_paragraph = Paragraph::new(error_text)
        .wrap(Wrap { trim: true })
        .block(block)
        .style(Style::default().fg(Color::LightRed));

    frame.render_widget(Clear, area);
    frame.render_widget(error_paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
