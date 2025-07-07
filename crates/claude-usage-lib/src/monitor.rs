use crate::calculator::Calculator;
use crate::data_structures::{BurnRate, ClaudePlan, SessionBlock, UsageEntry, UsageProjection};
use crate::identifier::SessionIdentifier;
use crate::loader::DataLoader;
use crate::pricing::PricingProvider;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

pub struct UsageMonitor {
    usage_entries: Vec<UsageEntry>,
    session_blocks: Vec<SessionBlock>,
    pricing_provider: PricingProvider,
    calculator: Calculator,
    identifier: SessionIdentifier,
    loader: DataLoader,
}

impl UsageMonitor {
    pub fn new() -> Self {
        Self {
            usage_entries: Vec::new(),
            session_blocks: Vec::new(),
            pricing_provider: PricingProvider::new(),
            calculator: Calculator::new(),
            identifier: SessionIdentifier::new(),
            loader: DataLoader::new(),
        }
    }

    pub fn load_data<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.usage_entries = self.loader.load_from_file(path)?;
        self.recalculate_blocks();
        Ok(())
    }

    pub fn load_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<()> {
        self.usage_entries = self.loader.load_from_directory(dir_path)?;
        self.recalculate_blocks();
        Ok(())
    }

    pub fn add_entry(&mut self, entry: UsageEntry) {
        self.usage_entries.push(entry);
        self.usage_entries
            .sort_by(|a, b| a.timestamp().cmp(&b.timestamp()));
        self.recalculate_blocks();
    }

    pub fn get_session_blocks(&self) -> &[SessionBlock] {
        &self.session_blocks
    }

    pub fn get_usage_entries(&self) -> &[UsageEntry] {
        &self.usage_entries
    }

    pub fn get_current_burn_rate(&self) -> Option<BurnRate> {
        self.session_blocks.last().and_then(|block| {
            self.calculator
                .calculate_weighted_burn_rate(block, &self.pricing_provider)
        })
    }

    pub fn get_burn_rate_for_block(&self, block_index: usize) -> Option<BurnRate> {
        self.session_blocks
            .get(block_index)
            .and_then(|block| self.calculator.calculate_burn_rate(block))
    }

    pub fn project_usage(
        &self,
        block_index: usize,
        current_time: DateTime<Utc>,
    ) -> Option<UsageProjection> {
        self.session_blocks
            .get(block_index)
            .and_then(|block| self.calculator.project_block_usage(block, current_time))
    }

    pub fn project_current_usage(&self, current_time: DateTime<Utc>) -> Option<UsageProjection> {
        if let Some(last_block_index) = self.session_blocks.len().checked_sub(1) {
            self.project_usage(last_block_index, current_time)
        } else {
            None
        }
    }

    pub fn calculate_hourly_burn_rate(&self, current_time: DateTime<Utc>) -> f64 {
        self.calculator.calculate_weighted_hourly_burn_rate(
            &self.session_blocks,
            current_time,
            &self.pricing_provider,
        )
    }

    pub fn calculate_tokens_per_second(&self, current_time: DateTime<Utc>) -> f64 {
        self.calculate_hourly_burn_rate(current_time) / 60.0
    }

    pub fn get_total_cost(&self) -> f64 {
        self.calculator.calculate_total_cost(&self.session_blocks)
    }

    pub fn get_total_tokens(&self) -> u64 {
        self.calculator.calculate_total_tokens(&self.session_blocks)
    }

    pub fn get_average_burn_rate(&self) -> Option<BurnRate> {
        self.calculator
            .calculate_average_burn_rate(&self.session_blocks)
    }

    pub fn get_peak_burn_rate(&self) -> Option<BurnRate> {
        self.calculator
            .calculate_peak_burn_rate(&self.session_blocks)
    }

    pub fn get_active_sessions(&self, current_time: DateTime<Utc>) -> Vec<&SessionBlock> {
        self.session_blocks
            .iter()
            .filter(|block| {
                !block.is_empty()
                    && current_time >= block.start_time()
                    && current_time < block.end_time()
            })
            .collect()
    }

    pub fn get_sessions_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&SessionBlock> {
        self.session_blocks
            .iter()
            .filter(|block| {
                !block.is_empty() && block.start_time() < end && block.end_time() > start
            })
            .collect()
    }

    pub fn get_model_breakdown(&self) -> std::collections::HashMap<String, (u64, f64)> {
        let mut breakdown = std::collections::HashMap::new();

        for entry in &self.usage_entries {
            let stats = breakdown
                .entry(entry.model().to_string())
                .or_insert((0, 0.0));
            stats.0 += entry.total_tokens();
            stats.1 += entry.cost_usd();
        }

        breakdown
    }

    pub fn get_weighted_tokens(&self, model: &str) -> f64 {
        let model_weight = self.pricing_provider.get_model_weight(model);
        self.usage_entries
            .iter()
            .filter(|entry| entry.model() == model)
            .map(|entry| {
                self.calculator
                    .calculate_weighted_tokens(entry, model_weight)
            })
            .sum()
    }

    pub fn get_total_weighted_tokens(&self) -> f64 {
        self.usage_entries
            .iter()
            .map(|entry| {
                let model_weight = self.pricing_provider.get_model_weight(entry.model());
                self.calculator
                    .calculate_weighted_tokens(entry, model_weight)
            })
            .sum()
    }

    pub fn estimate_time_to_limit(&self, token_limit: u64) -> Option<chrono::Duration> {
        let current_tokens = self.get_total_weighted_tokens() as u64;
        let current_burn_rate = self.get_current_burn_rate()?.tokens_per_minute();

        self.calculator
            .calculate_time_to_limit(current_tokens, token_limit, current_burn_rate)
    }

    pub fn estimate_time_to_plan_limit(&self, plan: ClaudePlan) -> Option<chrono::Duration> {
        self.estimate_time_to_limit(plan.max_tokens())
    }

    pub fn get_plan_usage_percentage(&self, plan: ClaudePlan) -> f64 {
        let current_tokens = self.get_total_weighted_tokens();
        (current_tokens / plan.max_tokens() as f64) * 100.0
    }

    pub fn get_supported_models(&self) -> Vec<&String> {
        self.pricing_provider.supported_models()
    }

    pub fn get_model_weight(&self, model: &str) -> f64 {
        self.pricing_provider.get_model_weight(model)
    }

    pub fn calculate_cost_for_tokens(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Option<f64> {
        self.pricing_provider
            .calculate_cost(model, input_tokens, output_tokens, 0, 0)
    }

    pub fn clear_data(&mut self) {
        self.usage_entries.clear();
        self.session_blocks.clear();
    }

    pub fn session_count(&self) -> usize {
        self.session_blocks.len()
    }

    pub fn entry_count(&self) -> usize {
        self.usage_entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.usage_entries.is_empty()
    }

    fn recalculate_blocks(&mut self) {
        self.session_blocks = self.identifier.identify_blocks(&self.usage_entries);
    }
}

impl Default for UsageMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::UsageEntry;
    use chrono::{Duration, TimeZone};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_monitor() {
        let monitor = UsageMonitor::new();
        assert!(monitor.is_empty());
        assert_eq!(monitor.session_count(), 0);
        assert_eq!(monitor.entry_count(), 0);
    }

    #[test]
    fn test_add_entry() {
        let mut monitor = UsageMonitor::new();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let entry = UsageEntry::new(
            timestamp,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );

        monitor.add_entry(entry);
        assert_eq!(monitor.entry_count(), 1);
        assert_eq!(monitor.session_count(), 1);
    }

    #[test]
    fn test_load_data_from_file() {
        let mut monitor = UsageMonitor::new();
        let mut temp_file = NamedTempFile::new().unwrap();

        let content = r#"{"timestamp": "2024-01-01T12:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 100, "output_tokens": 50, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}, "cost_usd": 0.001}"#;
        temp_file.write_all(content.as_bytes()).unwrap();

        monitor.load_data(temp_file.path()).unwrap();
        assert_eq!(monitor.entry_count(), 1);
        assert_eq!(monitor.session_count(), 1);
    }

    #[test]
    fn test_get_current_burn_rate() {
        let mut monitor = UsageMonitor::new();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let entry1 = UsageEntry::new(
            timestamp,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );
        let entry2 = UsageEntry::new(
            timestamp + Duration::minutes(1),
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );

        monitor.add_entry(entry1);
        monitor.add_entry(entry2);

        let burn_rate = monitor.get_current_burn_rate().unwrap();
        assert!(burn_rate.tokens_per_minute() > 0.0);
        assert!(burn_rate.cost_per_hour() > 0.0);
    }

    #[test]
    fn test_get_model_breakdown() {
        let mut monitor = UsageMonitor::new();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

        let entry1 = UsageEntry::new(
            timestamp,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );
        let entry2 = UsageEntry::new(
            timestamp + Duration::minutes(1),
            "claude-3-opus-20240229".to_string(),
            200,
            100,
            0,
            0,
            0.005,
        );

        monitor.add_entry(entry1);
        monitor.add_entry(entry2);

        let breakdown = monitor.get_model_breakdown();
        assert_eq!(breakdown.len(), 2);
        assert!(breakdown.contains_key("claude-3-sonnet-20240229"));
        assert!(breakdown.contains_key("claude-3-opus-20240229"));
    }

    #[test]
    fn test_clear_data() {
        let mut monitor = UsageMonitor::new();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let entry = UsageEntry::new(
            timestamp,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );

        monitor.add_entry(entry);
        assert!(!monitor.is_empty());

        monitor.clear_data();
        assert!(monitor.is_empty());
        assert_eq!(monitor.session_count(), 0);
        assert_eq!(monitor.entry_count(), 0);
    }
}
