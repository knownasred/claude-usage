use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Gauge},
    Frame,
};

use crate::AppState;

pub struct ProgressBarsWidget;

impl ProgressBarsWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let usage_percentage = state.get_usage_percentage();
        let token_gauge = Gauge::default()
            .block(Block::bordered().title("Token Usage"))
            .gauge_style(if usage_percentage > 80.0 {
                Style::default().fg(Color::Red)
            } else if usage_percentage > 60.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            })
            .percent(usage_percentage.min(100.0) as u16)
            .label(format!("{:.1}%", usage_percentage));

        frame.render_widget(token_gauge, chunks[0]);

        let (time_remaining, time_percentage) = state.get_time_to_reset_formatted();
        let elapsed_percentage = (1.0 - time_percentage) * 100.0;
        let time_gauge = Gauge::default()
            .block(Block::bordered().title("Session Time (5h blocks)"))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent(elapsed_percentage.max(0.0).min(100.0) as u16)
            .label(format!("{} remaining", time_remaining));

        frame.render_widget(time_gauge, chunks[1]);
    }
}
