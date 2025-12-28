use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use tui_input::Input;
use sentorii_contracts::ui::{UiState, UiStepStatus};

pub fn render(frame: &mut Frame, state: &mut UiState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(frame.size());

    let steps_list = state
        .steps()
        .iter()
        .map(|step| {
            let prefix = match step.state() {
                UiStepStatus::Pending => " [ ]",
                UiStepStatus::Running => " [*]",
                UiStepStatus::Success => " [✔]",
            };
            Line::from(format!("{} {}", prefix, step.description()))
        })
        .collect::<Vec<_>>();

    let steps_paragraph = Paragraph::new(steps_list)
        .block(
            Block::default()
                .title(" Sentorii Feature Start ")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(steps_paragraph, main_layout[0]);

    if state.is_awaiting_input() {
        if let Some(input) = state.input() {
            render_input_modal(frame, state.prompt(), input);
        }
    }

    if let Some(error_message) = state.error_message() {
        render_error_modal(frame, error_message);
    }
}

fn render_input_modal(frame: &mut Frame, prompt: &str, input: &Input) {
    let area = centered_rect(60, 20, frame.size());
    let block = Block::default()
        .title(prompt)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_paragraph = Paragraph::new(input.value()).block(block);

    frame.render_widget(Clear, area);
    frame.render_widget(input_paragraph, area);

    frame.set_cursor(
        area.x + input.visual_cursor() as u16 + 1,
        area.y + 1,
    );
}

fn render_error_modal(frame: &mut Frame, error_message: &str) {
    let area = centered_rect(80, 25, frame.size());
    let block = Block::default()
        .title(" Workflow Failed ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let error_text = format!(
        "{}\n\nPress any key to exit.",
        error_message
    );

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