use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::AppState;

pub struct PopupWidget;

impl PopupWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let popup_area = Self::centered_rect(60, 70, area);

        // Clear the area first
        frame.render_widget(Clear, popup_area);

        let debug_text = Self::create_debug_breakdown_text(state);

        let popup = Paragraph::new(debug_text)
            .block(
                Block::bordered()
                    .title("Current Block Breakdown")
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(popup, popup_area);
    }

    fn create_debug_breakdown_text(state: &AppState) -> Vec<Line> {
        let current_tokens = state.get_current_tokens();
        let current_cost = state.get_current_block_cost();
        let current_duration = state.get_current_block_duration();

        let mut debug_text = vec![
            Line::from(vec![
                Span::styled("Block Tokens: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", current_tokens),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Block Cost: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("${:.3}", current_cost),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Block Duration: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.1} min", current_duration),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(" "),
            Line::from(vec![Span::styled(
                "Model Breakdown:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(" "),
        ];

        // Add model breakdown (current block only)
        let model_breakdown = state.usage_monitor.get_current_block_model_breakdown();
        let mut sorted_models: Vec<_> = model_breakdown.iter().collect();
        sorted_models.sort_by(|a, b| a.0.cmp(b.0));

        for (model, (raw_tokens, _cost)) in sorted_models {
            let weight = state.usage_monitor.get_model_weight(model);
            let weighted_tokens = *raw_tokens as f64 * weight;

            let model_display = if model.contains("opus") {
                format!(
                    "Opus: {} → {} (×{})",
                    raw_tokens, weighted_tokens as u64, weight
                )
            } else if model.contains("sonnet") {
                format!(
                    "Sonnet: {} → {} (×{})",
                    raw_tokens, weighted_tokens as u64, weight
                )
            } else if model.contains("haiku") {
                format!(
                    "Haiku: {} → {} (×{})",
                    raw_tokens, weighted_tokens as u64, weight
                )
            } else {
                format!(
                    "{}: {} → {} (×{})",
                    model, raw_tokens, weighted_tokens as u64, weight
                )
            };

            debug_text.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(model_display, Style::default().fg(Color::White)),
            ]));
        }

        debug_text.extend(vec![
            Line::from(" "),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "d",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close", Style::default().fg(Color::Gray)),
            ]),
        ]);

        debug_text
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
                ratatui::layout::Constraint::Percentage(percent_y),
                ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
                ratatui::layout::Constraint::Percentage(percent_x),
                ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
