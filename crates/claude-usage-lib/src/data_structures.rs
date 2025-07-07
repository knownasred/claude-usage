use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudePlan {
    Pro,
    Max5,
    Max20,
}

impl ClaudePlan {
    pub fn max_tokens(&self) -> u64 {
        match self {
            ClaudePlan::Pro => 44_000,
            ClaudePlan::Max5 => 220_000,
            ClaudePlan::Max20 => 880_000,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ClaudePlan::Pro => "Claude Pro",
            ClaudePlan::Max5 => "Claude Max 5",
            ClaudePlan::Max20 => "Claude Max 20",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ClaudePlan::Pro => "Claude Pro (~44K tokens/day)",
            ClaudePlan::Max5 => "Claude Max 5 (~220K tokens/day)", 
            ClaudePlan::Max20 => "Claude Max 20 (~880K tokens/day)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    timestamp: DateTime<Utc>,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_input_tokens: u64,
    cache_read_input_tokens: u64,
    cost_usd: f64,
}

impl UsageEntry {
    pub fn new(
        timestamp: DateTime<Utc>,
        model: String,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
        cost_usd: f64,
    ) -> Self {
        Self {
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            cost_usd,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn input_tokens(&self) -> u64 {
        self.input_tokens
    }

    pub fn output_tokens(&self) -> u64 {
        self.output_tokens
    }

    pub fn cache_creation_input_tokens(&self) -> u64 {
        self.cache_creation_input_tokens
    }

    pub fn cache_read_input_tokens(&self) -> u64 {
        self.cache_read_input_tokens
    }

    pub fn cost_usd(&self) -> f64 {
        self.cost_usd
    }

    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn all_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_input_tokens + self.cache_read_input_tokens
    }
}

#[derive(Debug, Clone)]
pub struct TokenCounts {
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_input_tokens: u64,
    cache_read_input_tokens: u64,
}

impl TokenCounts {
    pub fn new() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        }
    }

    pub fn add_entry(&mut self, entry: &UsageEntry) {
        self.input_tokens += entry.input_tokens;
        self.output_tokens += entry.output_tokens;
        self.cache_creation_input_tokens += entry.cache_creation_input_tokens;
        self.cache_read_input_tokens += entry.cache_read_input_tokens;
    }

    pub fn input_tokens(&self) -> u64 {
        self.input_tokens
    }

    pub fn output_tokens(&self) -> u64 {
        self.output_tokens
    }

    pub fn cache_creation_input_tokens(&self) -> u64 {
        self.cache_creation_input_tokens
    }

    pub fn cache_read_input_tokens(&self) -> u64 {
        self.cache_read_input_tokens
    }

    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn all_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_input_tokens + self.cache_read_input_tokens
    }
}

#[derive(Debug, Clone)]
pub struct SessionBlock {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    entries: Vec<UsageEntry>,
    token_counts: TokenCounts,
    cost_usd: f64,
    duration_minutes: f64,
}

impl SessionBlock {
    pub fn new(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        Self {
            start_time,
            end_time,
            entries: Vec::new(),
            token_counts: TokenCounts::new(),
            cost_usd: 0.0,
            duration_minutes: 0.0,
        }
    }

    pub fn add_entry(&mut self, entry: UsageEntry) {
        self.token_counts.add_entry(&entry);
        self.cost_usd += entry.cost_usd;
        self.entries.push(entry);
        self.update_duration();
    }

    pub fn update_duration(&mut self) {
        if !self.entries.is_empty() {
            let actual_start = self.entries.first().unwrap().timestamp;
            let actual_end = self.entries.last().unwrap().timestamp;
            self.duration_minutes = (actual_end - actual_start).num_seconds() as f64 / 60.0;
            
            if self.duration_minutes == 0.0 {
                self.duration_minutes = 1.0;
            }
        }
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn end_time(&self) -> DateTime<Utc> {
        self.end_time
    }

    pub fn entries(&self) -> &[UsageEntry] {
        &self.entries
    }

    pub fn token_counts(&self) -> &TokenCounts {
        &self.token_counts
    }

    pub fn cost_usd(&self) -> f64 {
        self.cost_usd
    }

    pub fn duration_minutes(&self) -> f64 {
        self.duration_minutes
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct BurnRate {
    tokens_per_minute: f64,
    cost_per_hour: f64,
}

impl BurnRate {
    pub fn new(tokens_per_minute: f64, cost_per_hour: f64) -> Self {
        Self {
            tokens_per_minute,
            cost_per_hour,
        }
    }

    pub fn tokens_per_minute(&self) -> f64 {
        self.tokens_per_minute
    }

    pub fn cost_per_hour(&self) -> f64 {
        self.cost_per_hour
    }

    pub fn tokens_per_second(&self) -> f64 {
        self.tokens_per_minute / 60.0
    }
}

#[derive(Debug, Clone)]
pub struct UsageProjection {
    current_tokens: u64,
    current_cost: f64,
    projected_additional_tokens: u64,
    projected_additional_cost: f64,
    projected_total_tokens: u64,
    projected_total_cost: f64,
}

impl UsageProjection {
    pub fn new(
        current_tokens: u64,
        current_cost: f64,
        projected_additional_tokens: u64,
        projected_additional_cost: f64,
    ) -> Self {
        Self {
            current_tokens,
            current_cost,
            projected_additional_tokens,
            projected_additional_cost,
            projected_total_tokens: current_tokens + projected_additional_tokens,
            projected_total_cost: current_cost + projected_additional_cost,
        }
    }

    pub fn current_tokens(&self) -> u64 {
        self.current_tokens
    }

    pub fn current_cost(&self) -> f64 {
        self.current_cost
    }

    pub fn projected_additional_tokens(&self) -> u64 {
        self.projected_additional_tokens
    }

    pub fn projected_additional_cost(&self) -> f64 {
        self.projected_additional_cost
    }

    pub fn projected_total_tokens(&self) -> u64 {
        self.projected_total_tokens
    }

    pub fn projected_total_cost(&self) -> f64 {
        self.projected_total_cost
    }
}

#[derive(Debug, Clone)]
pub struct ModelPricing {
    input_cost_per_token: f64,
    output_cost_per_token: f64,
    cache_creation_input_token_cost: f64,
    cache_read_input_token_cost: f64,
}

impl ModelPricing {
    pub fn new(
        input_cost_per_token: f64,
        output_cost_per_token: f64,
        cache_creation_input_token_cost: f64,
        cache_read_input_token_cost: f64,
    ) -> Self {
        Self {
            input_cost_per_token,
            output_cost_per_token,
            cache_creation_input_token_cost,
            cache_read_input_token_cost,
        }
    }

    pub fn input_cost_per_token(&self) -> f64 {
        self.input_cost_per_token
    }

    pub fn output_cost_per_token(&self) -> f64 {
        self.output_cost_per_token
    }

    pub fn cache_creation_input_token_cost(&self) -> f64 {
        self.cache_creation_input_token_cost
    }

    pub fn cache_read_input_token_cost(&self) -> f64 {
        self.cache_read_input_token_cost
    }

    pub fn calculate_cost(
        &self,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
    ) -> f64 {
        let cost = (input_tokens as f64 * self.input_cost_per_token)
            + (output_tokens as f64 * self.output_cost_per_token)
            + (cache_creation_tokens as f64 * self.cache_creation_input_token_cost)
            + (cache_read_tokens as f64 * self.cache_read_input_token_cost);
        
        (cost * 1_000_000.0).round() / 1_000_000.0
    }
}