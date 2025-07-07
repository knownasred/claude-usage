use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::AppState;

pub struct StatisticsWidget;

impl StatisticsWidget {
    pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
        let current_tokens = state.get_current_tokens();
        let burn_rate = state.get_burn_rate();

        let mut stats_text = vec![
            Line::from(vec![
                Span::styled("Data Status: ", Style::default().fg(Color::White)),
                Span::styled(
                    if state.data_loaded {
                        format!("Loaded ({} entries)", state.usage_monitor.entry_count())
                    } else if state.is_loading {
                        "Loading...".to_string()
                    } else {
                        "No data".to_string()
                    },
                    Style::default()
                        .fg(if state.data_loaded {
                            Color::Green
                        } else {
                            Color::Red
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", current_tokens),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" / {}", state.plan.max_tokens()),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::styled("Burn Rate: ", Style::default().fg(Color::White)),
                Span::styled(
                    if let Some(br) = burn_rate {
                        format!("{:.1} tokens/min", br.tokens_per_minute())
                    } else {
                        "N/A".to_string()
                    },
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        if let Some(error) = &state.error_message {
            stats_text.push(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red)),
                Span::styled(
                    error.chars().take(50).collect::<String>()
                        + if error.len() > 50 { "..." } else { "" },
                    Style::default().fg(Color::Red),
                ),
            ]));
        } else {
            stats_text.push(Line::from(vec![
                Span::styled("Last Update: ", Style::default().fg(Color::White)),
                Span::styled(
                    state.last_update.format("%H:%M:%S UTC").to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }

        let stats = Paragraph::new(stats_text)
            .block(Block::bordered().title("Statistics"))
            .alignment(Alignment::Left);

        frame.render_widget(stats, area);
    }
}
