use claude_usage_monitor::prelude::*;
use std::env;
use std::path::Path;

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
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

fn main() -> Result<()> {
    let mut monitor = UsageMonitor::new();

    let args: Vec<String> = env::args().collect();

    // Check if a specific file path was provided
    if args.len() >= 2 {
        let file_path = &args[1];
        println!("Loading usage data from: {}", file_path);
        monitor.load_data(file_path)?;
    } else {
        // Auto-discover Claude data paths
        println!("Auto-discovering Claude usage data...");
        let claude_paths = discover_claude_data_paths();

        if claude_paths.is_empty() {
            eprintln!("No Claude data directories found in standard locations:");
            eprintln!("  ~/.claude/projects");
            eprintln!("  ~/.config/claude/projects");
            eprintln!();
            eprintln!("Usage: {} [path_to_usage_file.jsonl]", args[0]);
            std::process::exit(1);
        }

        let mut loaded_any = false;
        for claude_path in &claude_paths {
            println!("Checking directory: {}", claude_path.display());
            match monitor.load_directory(claude_path) {
                Ok(_) => {
                    if !monitor.is_empty() {
                        loaded_any = true;
                        println!("Successfully loaded data from: {}", claude_path.display());
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load from {}: {}",
                        claude_path.display(),
                        e
                    );
                }
            }
        }

        if !loaded_any {
            eprintln!("No usage data found in any Claude directories.");
            eprintln!("Make sure you have Claude usage data in one of:");
            for path in &claude_paths {
                eprintln!("  {}", path.display());
            }
            std::process::exit(1);
        }
    }

    println!(
        "Loaded {} entries across {} sessions",
        monitor.entry_count(),
        monitor.session_count()
    );

    if monitor.is_empty() {
        println!("No usage data found.");
        return Ok(());
    }

    println!("\n--- Overall Statistics ---");
    println!("Total tokens: {}", monitor.get_total_tokens());
    println!("Total cost: ${:.6}", monitor.get_total_cost());
    println!(
        "Total weighted tokens: {:.2}",
        monitor.get_total_weighted_tokens()
    );

    if let Some(avg_burn_rate) = monitor.get_average_burn_rate() {
        println!(
            "Average burn rate: {:.2} tokens/minute, ${:.4}/hour",
            avg_burn_rate.tokens_per_minute(),
            avg_burn_rate.cost_per_hour()
        );
    }

    if let Some(peak_burn_rate) = monitor.get_peak_burn_rate() {
        println!(
            "Peak burn rate: {:.2} tokens/minute, ${:.4}/hour",
            peak_burn_rate.tokens_per_minute(),
            peak_burn_rate.cost_per_hour()
        );
    }

    let current_time = Utc::now();
    let hourly_rate = monitor.calculate_hourly_burn_rate(current_time);
    let tokens_per_second = monitor.calculate_tokens_per_second(current_time);

    println!("\n--- Current Rates ---");
    println!("Hourly burn rate: {:.2} tokens/minute", hourly_rate);
    println!("Tokens per second: {:.4}", tokens_per_second);

    if let Some(current_burn_rate) = monitor.get_current_burn_rate() {
        println!(
            "Current session burn rate: {:.2} tokens/minute, ${:.4}/hour",
            current_burn_rate.tokens_per_minute(),
            current_burn_rate.cost_per_hour()
        );
    }

    if let Some(projection) = monitor.project_current_usage(current_time) {
        println!("\n--- Current Session Projection ---");
        println!("Current tokens: {}", projection.current_tokens());
        println!("Current cost: ${:.6}", projection.current_cost());
        println!(
            "Projected additional tokens: {}",
            projection.projected_additional_tokens()
        );
        println!(
            "Projected additional cost: ${:.6}",
            projection.projected_additional_cost()
        );
        println!(
            "Projected total tokens: {}",
            projection.projected_total_tokens()
        );
        println!(
            "Projected total cost: ${:.6}",
            projection.projected_total_cost()
        );
    }

    println!("\n--- Model Breakdown ---");
    let breakdown = monitor.get_model_breakdown();
    for (model, (tokens, cost)) in breakdown {
        println!("{}: {} tokens, ${:.6}", model, tokens, cost);
    }

    println!("\n--- Supported Models ---");
    let models = monitor.get_supported_models();
    for model in models {
        println!("- {}", model);
    }

    println!("\n--- Claude Plan Usage Analysis ---");

    let plans = [ClaudePlan::Pro, ClaudePlan::Max5, ClaudePlan::Max20];
    let current_tokens = monitor.get_total_tokens();

    // Auto-detect most likely plan based on current usage
    let detected_plan = if current_tokens > ClaudePlan::Max5.max_tokens() {
        ClaudePlan::Max20
    } else if current_tokens > ClaudePlan::Pro.max_tokens() {
        ClaudePlan::Max5
    } else {
        ClaudePlan::Pro
    };

    println!(
        "Auto-detected plan: {} (based on current usage)",
        detected_plan.description()
    );
    println!();

    for plan in plans {
        let percentage = monitor.get_plan_usage_percentage(plan);
        let max_tokens = plan.max_tokens();

        println!("{}:", plan.description());

        if current_tokens < max_tokens {
            if let Some(time_to_limit) = monitor.estimate_time_to_plan_limit(plan) {
                let hours = time_to_limit.num_hours();
                let minutes = time_to_limit.num_minutes() % 60;
                let days = hours / 24;
                let remaining_hours = hours % 24;

                // Calculate when we'll hit the limit
                let current_time = Utc::now();
                let limit_time = current_time + time_to_limit;

                if days > 0 {
                    println!(
                        "  Time remaining: {}d {}h {}m",
                        days, remaining_hours, minutes
                    );
                } else if hours > 0 {
                    println!("  Time remaining: {}h {}m", hours, minutes);
                } else {
                    println!("  Time remaining: {}m", minutes);
                }
                println!(
                    "  Will be reached at: {}",
                    limit_time.format("%Y-%m-%d %H:%M UTC")
                );
            }
        } else {
            println!("  Status: EXCEEDED ({:.1}% over limit)", percentage - 100.0);
        }

        // Progress bar
        let bar_length = 20;
        let filled = ((percentage / 100.0) * bar_length as f64) as usize;
        let bar = if percentage > 100.0 {
            "█".repeat(bar_length) // Full red bar if exceeded
        } else {
            "█".repeat(filled.min(bar_length)) + &"░".repeat(bar_length - filled.min(bar_length))
        };

        let status_indicator = if plan == detected_plan {
            " ← DETECTED"
        } else {
            ""
        };
        println!(
            "  Usage: [{}] {:.1}% ({}/{}){}",
            bar,
            percentage,
            format_number(current_tokens),
            format_number(max_tokens),
            status_indicator
        );
        println!();
    }

    Ok(())
}
