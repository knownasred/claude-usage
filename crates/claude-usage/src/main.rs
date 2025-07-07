use anyhow::Result;
use chrono::{DateTime, Timelike, Utc};
use clap::Parser;
use claude_usage_monitor::prelude::*;
use claude_usage_monitor::{ClaudePlan, UsageMonitor};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;

mod widgets;
use widgets::*;

#[derive(Debug, Clone, PartialEq)]
pub enum PopupType {
    CurrentBlock,
    LifetimeStats,
}

#[derive(Parser, Debug)]
#[clap(author = "Red", version, about)]
struct Args {
    #[arg(short = 'v')]
    verbose: bool,

    #[arg(short = 'p', long = "plan", default_value = "pro")]
    plan: String,

    #[arg(short = 'd', long = "data-dir")]
    data_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UsageConfig {
    plan: String,
}

fn get_config_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let parent_dir = current_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot find parent directory"))?;
    Ok(parent_dir.join(".usage.json"))
}

fn load_config() -> Result<UsageConfig> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: UsageConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        // Return default config
        Ok(UsageConfig {
            plan: "pro".to_string(),
        })
    }
}

fn save_config(config: &UsageConfig) -> Result<()> {
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}

fn discover_claude_data_paths() -> Vec<std::path::PathBuf> {
    let standard_paths = ["~/.claude/projects", "~/.config/claude/projects"];

    let mut discovered_paths = Vec::new();

    for path_str in &standard_paths {
        let path = shellexpand::tilde(path_str);
        let path = Path::new(path.as_ref());
        if path.exists() && path.is_dir() {
            discovered_paths.push(path.to_path_buf());
        }
    }

    discovered_paths
}

pub struct AppState {
    pub usage_monitor: UsageMonitor,
    pub plan: ClaudePlan,
    pub last_update: DateTime<Utc>,
    pub is_loading: bool,
    pub spinner_state: usize,
    pub data_loaded: bool,
    pub error_message: Option<String>,
    pub active_popup: Option<PopupType>,
}

impl AppState {
    fn new(plan: ClaudePlan) -> Self {
        Self {
            usage_monitor: UsageMonitor::new(),
            plan,
            last_update: Utc::now(),
            is_loading: false,
            spinner_state: 0,
            data_loaded: false,
            error_message: None,
            active_popup: None,
        }
    }

    fn load_data(&mut self, data_dir: Option<String>) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;

        let result = if let Some(data_path) = data_dir {
            // Load from specific directory/file
            let path = Path::new(&data_path);
            if path.is_file() {
                self.usage_monitor.load_data(&data_path)
            } else if path.is_dir() {
                self.usage_monitor.load_directory(&data_path)
            } else {
                return Err(anyhow::anyhow!("Path does not exist: {}", data_path));
            }
        } else {
            // Auto-discover Claude data paths
            let claude_paths = discover_claude_data_paths();

            if claude_paths.is_empty() {
                return Err(anyhow::anyhow!(
                    "No Claude data directories found in standard locations:\n  ~/.claude/projects\n  ~/.config/claude/projects"
                ));
            }

            let mut loaded_any = false;
            let mut last_error = None;

            for claude_path in &claude_paths {
                match self.usage_monitor.load_directory(claude_path) {
                    Ok(_) => {
                        if !self.usage_monitor.is_empty() {
                            loaded_any = true;
                            break;
                        }
                    }
                    Err(e) => {
                        last_error = Some(e);
                    }
                }
            }

            if !loaded_any {
                if let Some(e) = last_error {
                    return Err(e);
                } else {
                    return Err(anyhow::anyhow!(
                        "No usage data found in any Claude directories"
                    ));
                }
            }

            Ok(())
        };

        match &result {
            Ok(_) => {
                self.data_loaded = true;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
                self.data_loaded = false;
            }
        }

        self.is_loading = false;
        self.last_update = Utc::now();

        result
    }

    fn update_spinner(&mut self) {
        self.spinner_state = (self.spinner_state + 1) % 10;
    }

    pub fn get_spinner_char(&self) -> char {
        match self.spinner_state {
            0 => '⠋',
            1 => '⠙',
            2 => '⠹',
            3 => '⠸',
            4 => '⠼',
            5 => '⠴',
            6 => '⠦',
            7 => '⠧',
            8 => '⠇',
            9 => '⠏',
            _ => '⠋',
        }
    }

    pub fn get_usage_percentage(&self) -> f64 {
        self.usage_monitor.get_current_block_percentage(self.plan)
    }

    pub fn get_current_tokens(&self) -> u64 {
        self.usage_monitor.get_current_block_tokens() as u64
    }

    pub fn get_burn_rate(&self) -> Option<BurnRate> {
        self.usage_monitor.get_current_burn_rate()
    }

    // Lifetime stats for future popup
    pub fn get_lifetime_tokens(&self) -> u64 {
        self.usage_monitor.get_total_weighted_tokens() as u64
    }

    pub fn get_lifetime_percentage(&self, plan: ClaudePlan) -> f64 {
        self.usage_monitor.get_plan_usage_percentage(plan)
    }

    pub fn get_total_cost(&self) -> f64 {
        self.usage_monitor.get_total_cost()
    }

    // Current block methods for debug popup
    pub fn get_current_block_cost(&self) -> f64 {
        self.usage_monitor.get_current_block_cost()
    }

    pub fn get_current_block_duration(&self) -> f64 {
        self.usage_monitor.get_current_block_duration()
    }

    // Additional lifetime stats for popup
    pub fn get_session_blocks_count(&self) -> usize {
        self.usage_monitor.session_count()
    }

    pub fn get_average_burn_rate(&self) -> Option<BurnRate> {
        self.usage_monitor.get_average_burn_rate()
    }

    pub fn get_peak_burn_rate(&self) -> Option<BurnRate> {
        self.usage_monitor.get_peak_burn_rate()
    }

    pub fn get_time_to_reset_formatted(&self) -> (String, f64) {
        let now = Utc::now();

        // Get the current 5-hour session block
        if let Some(current_block) = self.usage_monitor.get_session_blocks().last() {
            if !current_block.is_empty() && now < current_block.end_time() {
                // We're in an active session block
                let remaining = current_block.end_time() - now;
                let total_seconds = remaining.num_seconds().max(0);
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;

                // Calculate percentage based on 5-hour session
                let elapsed = now - current_block.start_time();
                let session_duration = chrono::Duration::hours(5).num_seconds();
                let elapsed_percentage =
                    1.0 - (elapsed.num_seconds() as f64 / session_duration as f64);

                return (
                    format!("{}:{:02}", hours, minutes),
                    elapsed_percentage.min(1.0),
                );
            }
        }

        // No active session or no data - calculate next 5-hour window
        let current_hour = now
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let next_session_start = current_hour + chrono::Duration::hours(1);
        let next_session_end = next_session_start + chrono::Duration::hours(5);

        let remaining = next_session_end - now;
        let total_seconds = remaining.num_seconds().max(0);
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;

        // For inactive session, show as 0% elapsed
        (format!("{}:{:02}", hours, minutes), 0.0)
    }
}

pub struct App {
    state: Arc<Mutex<AppState>>,
    exit: bool,
}

impl App {
    pub fn new(plan: ClaudePlan, data_dir: Option<String>) -> Self {
        let mut app_state = AppState::new(plan);

        // Try to load data initially
        if let Err(e) = app_state.load_data(data_dir.clone()) {
            app_state.error_message = Some(format!("Initial load failed: {}", e));
        }

        Self {
            state: Arc::new(Mutex::new(app_state)),
            exit: false,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        data_dir: Option<String>,
    ) -> Result<()> {
        let state_clone = Arc::clone(&self.state);
        let data_dir_clone = data_dir.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;

                if let Ok(mut state) = state_clone.lock() {
                    // Reload data every 5 seconds
                    let _ = state.load_data(data_dir_clone.clone());
                }
            }
        });

        let mut tick_interval = interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    if let Ok(mut state) = self.state.lock() {
                        state.update_spinner();
                    }

                    terminal.draw(|frame| self.draw(frame))?;
                }

                _ = async {
                    if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                        if let Ok(event) = event::read() {
                            self.handle_event(event);
                        }
                    }
                } => {}
            }

            if self.exit {
                break;
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(area);

        if let Ok(state) = self.state.lock() {
            HeaderWidget::render(frame, chunks[0], &state);
            ProgressBarsWidget::render(frame, chunks[1], &state);
            StatisticsWidget::render(frame, chunks[2], &state);
            PredictionsWidget::render(frame, chunks[3], &state);
            ShortcutsWidget::render(frame, chunks[4], &state);

            // Render popup based on active popup type
            match &state.active_popup {
                Some(PopupType::CurrentBlock) => {
                    PopupWidget::render(frame, area, &state);
                }
                Some(PopupType::LifetimeStats) => {
                    LifetimePopupWidget::render(frame, area, &state);
                }
                None => {}
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('r') => {
                        // Manual refresh - note: data_dir needs to be stored in app state for this to work
                        // For now, just mark as loading to trigger background reload
                        if let Ok(mut state) = self.state.lock() {
                            state.is_loading = true;
                        }
                    }
                    KeyCode::Char('d') => {
                        // Toggle current block breakdown popup
                        if let Ok(mut state) = self.state.lock() {
                            state.active_popup =
                                if state.active_popup == Some(PopupType::CurrentBlock) {
                                    None
                                } else {
                                    Some(PopupType::CurrentBlock)
                                };
                        }
                    }
                    KeyCode::Char('s') => {
                        // Toggle lifetime stats popup
                        if let Ok(mut state) = self.state.lock() {
                            state.active_popup =
                                if state.active_popup == Some(PopupType::LifetimeStats) {
                                    None
                                } else {
                                    Some(PopupType::LifetimeStats)
                                };
                        }
                    }
                    KeyCode::Esc => {
                        // Close any popup if open
                        if let Ok(mut state) = self.state.lock() {
                            state.active_popup = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load config and determine the plan to use
    let mut config = load_config().unwrap_or_else(|_| UsageConfig {
        plan: "pro".to_string(),
    });

    // If plan was specified via command line, use it and save it
    let plan_str =
        if args.plan != "pro" || std::env::args().any(|arg| arg == "--plan" || arg == "-p") {
            // Plan was explicitly specified, update config
            config.plan = args.plan.clone();
            if let Err(e) = save_config(&config) {
                eprintln!("Warning: Could not save config: {}", e);
            }
            args.plan.as_str()
        } else {
            // Use plan from config
            config.plan.as_str()
        };

    let plan = match plan_str {
        "pro" => ClaudePlan::Pro,
        "max5" => ClaudePlan::Max5,
        "max20" => ClaudePlan::Max20,
        _ => ClaudePlan::Pro,
    };

    let mut terminal = ratatui::init();
    let mut app = App::new(plan, args.data_dir.clone());

    let result = app.run(&mut terminal, args.data_dir).await;

    ratatui::restore();

    result
}
