use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::AppState;

pub struct PredictionsWidget;

impl PredictionsWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let burn_rate = state.get_burn_rate();
        let current_tokens = state.get_current_tokens();

        let predictions_text = if let Some(br) = burn_rate {
            let remaining_tokens = state.plan.max_tokens().saturating_sub(current_tokens);
            let minutes_to_limit = if br.tokens_per_minute() > 0.0 {
                remaining_tokens as f64 / br.tokens_per_minute()
            } else {
                0.0
            };

            let hours = minutes_to_limit / 60.0;
            let (time_to_reset_formatted, _) = state.get_time_to_reset_formatted();

            let predictions_text = vec![
                Line::from(vec![
                    Span::styled(
                        "Estimated time to limit: ",
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        if hours > 0.0 {
                            format!("{:.1} hours", hours)
                        } else {
                            "Limit reached".to_string()
                        },
                        Style::default()
                            .fg(if hours < 1.0 {
                                Color::Red
                            } else {
                                Color::Green
                            })
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Session time remaining: ",
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        time_to_reset_formatted,
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ];

            predictions_text
        } else {
            Self::render_no_data_text(state)
        };

        let predictions = Paragraph::new(predictions_text)
            .block(Block::bordered().title("Predictions"))
            .alignment(Alignment::Left);

        frame.render_widget(predictions, area);
    }

    fn render_no_data_text(state: &AppState) -> Vec<Line> {
        let mut no_data_text = vec![
            Line::from(vec![Span::styled(
                if state.data_loaded {
                    "No usage data in loaded files"
                } else {
                    "No Claude usage data found"
                },
                Style::default().fg(Color::Red),
            )]),
            Line::from(" "),
        ];

        if !state.data_loaded {
            no_data_text.push(Line::from(vec![Span::styled(
                "Searched in:",
                Style::default().fg(Color::Gray),
            )]));
            no_data_text.push(Line::from(vec![Span::styled(
                "  ~/.claude/projects",
                Style::default().fg(Color::Gray),
            )]));
            no_data_text.push(Line::from(vec![Span::styled(
                "  ~/.config/claude/projects",
                Style::default().fg(Color::Gray),
            )]));
            no_data_text.push(Line::from(" "));
        }

        no_data_text
    }
}
