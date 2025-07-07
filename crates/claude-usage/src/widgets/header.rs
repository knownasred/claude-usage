use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::AppState;

pub struct HeaderWidget;

impl HeaderWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let spinner = if state.is_loading {
            state.get_spinner_char().to_string()
        } else {
            " ".to_string()
        };

        let title = format!("Claude Usage Monitor - {}", state.plan.name());
        let header_text = vec![Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(spinner, Style::default().fg(Color::Yellow)),
        ])];

        let header = Paragraph::new(header_text)
            .block(Block::bordered().title("Status"))
            .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }
}
