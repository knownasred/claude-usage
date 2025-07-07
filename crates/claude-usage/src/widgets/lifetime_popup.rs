use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::AppState;

pub struct LifetimePopupWidget;

impl LifetimePopupWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let popup_area = Self::centered_rect(60, 80, area);

        // Clear the area first
        frame.render_widget(Clear, popup_area);

        let lifetime_text = Self::create_lifetime_stats_text(state);

        let popup = Paragraph::new(lifetime_text)
            .block(
                Block::bordered()
                    .title("Session Statistics")
                    .title_alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(popup, popup_area);
    }

    fn create_lifetime_stats_text(state: &AppState) -> Vec<Line> {
        let lifetime_tokens = state.get_lifetime_tokens();
        let lifetime_percentage = state.get_lifetime_percentage(state.plan);
        let total_cost = state.get_total_cost();
        let blocks_count = state.get_session_blocks_count();
        let avg_burn_rate = state.get_average_burn_rate();
        let peak_burn_rate = state.get_peak_burn_rate();

        let mut lifetime_text = vec![
            Line::from(vec![
                Span::styled("Total Tokens: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", lifetime_tokens),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Usage: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.1}%", lifetime_percentage),
                    Style::default()
                        .fg(if lifetime_percentage > 80.0 {
                            Color::Red
                        } else if lifetime_percentage > 60.0 {
                            Color::Yellow
                        } else {
                            Color::Green
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Cost: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("${:.3}", total_cost),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Session Blocks: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", blocks_count),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        // Add burn rate information
        if let Some(avg_br) = avg_burn_rate {
            lifetime_text.push(Line::from(vec![
                Span::styled("Average Burn Rate: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.1} tokens/min", avg_br.tokens_per_minute()),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        if let Some(peak_br) = peak_burn_rate {
            lifetime_text.push(Line::from(vec![
                Span::styled("Peak Burn Rate: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:.1} tokens/min", peak_br.tokens_per_minute()),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        lifetime_text.extend(vec![
            Line::from(" "),
            Line::from(vec![Span::styled(
                "Model Breakdown (Lifetime):",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(" "),
        ]);

        // Add lifetime model breakdown
        let model_breakdown = state.usage_monitor.get_model_breakdown();
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

            lifetime_text.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(model_display, Style::default().fg(Color::White)),
            ]));
        }

        lifetime_text.extend(vec![
            Line::from(" "),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "s",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close", Style::default().fg(Color::Gray)),
            ]),
        ]);

        lifetime_text
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
