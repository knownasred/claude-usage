use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::AppState;

pub struct ShortcutsWidget;

impl ShortcutsWidget {
    pub fn render(frame: &mut Frame, area: Rect, _state: &AppState) {
        let shortcuts_text = vec![Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to quit, ", Style::default().fg(Color::Gray)),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to refresh, ", Style::default().fg(Color::Gray)),
            Span::styled(
                "d",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for debug", Style::default().fg(Color::Gray)),
        ])];

        let shortcuts = Paragraph::new(shortcuts_text).alignment(Alignment::Center);

        frame.render_widget(shortcuts, area);
    }
}
