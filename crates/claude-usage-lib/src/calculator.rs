use crate::data_structures::{BurnRate, SessionBlock, UsageEntry, UsageProjection};
use chrono::{DateTime, Duration, Utc};

pub struct Calculator;

impl Calculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_burn_rate(&self, block: &SessionBlock) -> Option<BurnRate> {
        if block.is_empty() || block.duration_minutes() == 0.0 {
            return None;
        }

        let total_tokens = block.token_counts().total_tokens() as f64;
        let tokens_per_minute = total_tokens / block.duration_minutes();
        let cost_per_hour = (block.cost_usd() / block.duration_minutes()) * 60.0;

        Some(BurnRate::new(tokens_per_minute, cost_per_hour))
    }

    pub fn project_block_usage(
        &self,
        block: &SessionBlock,
        current_time: DateTime<Utc>,
    ) -> Option<UsageProjection> {
        if block.is_empty() || current_time >= block.end_time() {
            return None;
        }

        let burn_rate = self.calculate_burn_rate(block)?;
        let current_tokens = block.token_counts().total_tokens();
        let current_cost = block.cost_usd();

        let remaining_duration = block.end_time() - current_time;
        let remaining_minutes = remaining_duration.num_seconds() as f64 / 60.0;
        let remaining_hours = remaining_minutes / 60.0;

        let projected_additional_tokens =
            (burn_rate.tokens_per_minute() * remaining_minutes) as u64;
        let projected_additional_cost = burn_rate.cost_per_hour() * remaining_hours;

        Some(UsageProjection::new(
            current_tokens,
            current_cost,
            projected_additional_tokens,
            projected_additional_cost,
        ))
    }

    pub fn calculate_hourly_burn_rate(
        &self,
        blocks: &[SessionBlock],
        current_time: DateTime<Utc>,
    ) -> f64 {
        let one_hour_ago = current_time - Duration::hours(1);
        let mut total_tokens = 0.0;

        for block in blocks {
            if block.is_empty() {
                continue;
            }

            let block_start = block.entries().first().unwrap().timestamp();
            let block_end = block.entries().last().unwrap().timestamp();

            if block_end < one_hour_ago || block_start > current_time {
                continue;
            }

            let overlap_start = block_start.max(one_hour_ago);
            let overlap_end = block_end.min(current_time);
            let overlap_duration = (overlap_end - overlap_start).num_seconds() as f64 / 60.0;

            if overlap_duration > 0.0 {
                let session_tokens = block.token_counts().total_tokens() as f64;
                let total_session_duration = block.duration_minutes();

                if total_session_duration > 0.0 {
                    let tokens_in_hour =
                        session_tokens * (overlap_duration / total_session_duration);
                    total_tokens += tokens_in_hour;
                }
            }
        }

        total_tokens / 60.0
    }

    pub fn calculate_weighted_tokens(&self, entry: &UsageEntry, model_weight: f64) -> f64 {
        (entry.total_tokens() as f64) * model_weight
    }

    pub fn calculate_total_cost(&self, blocks: &[SessionBlock]) -> f64 {
        blocks.iter().map(|block| block.cost_usd()).sum()
    }

    pub fn calculate_total_tokens(&self, blocks: &[SessionBlock]) -> u64 {
        blocks
            .iter()
            .map(|block| block.token_counts().total_tokens())
            .sum()
    }

    pub fn calculate_average_burn_rate(&self, blocks: &[SessionBlock]) -> Option<BurnRate> {
        if blocks.is_empty() {
            return None;
        }

        let burn_rates: Vec<BurnRate> = blocks
            .iter()
            .filter_map(|block| self.calculate_burn_rate(block))
            .collect();

        if burn_rates.is_empty() {
            return None;
        }

        let avg_tokens_per_minute = burn_rates
            .iter()
            .map(|br| br.tokens_per_minute())
            .sum::<f64>()
            / burn_rates.len() as f64;

        let avg_cost_per_hour =
            burn_rates.iter().map(|br| br.cost_per_hour()).sum::<f64>() / burn_rates.len() as f64;

        Some(BurnRate::new(avg_tokens_per_minute, avg_cost_per_hour))
    }

    pub fn calculate_peak_burn_rate(&self, blocks: &[SessionBlock]) -> Option<BurnRate> {
        blocks
            .iter()
            .filter_map(|block| self.calculate_burn_rate(block))
            .max_by(|a, b| {
                a.tokens_per_minute()
                    .partial_cmp(&b.tokens_per_minute())
                    .unwrap()
            })
    }

    pub fn calculate_time_to_limit(
        &self,
        current_tokens: u64,
        token_limit: u64,
        current_burn_rate: f64,
    ) -> Option<Duration> {
        if current_tokens >= token_limit || current_burn_rate <= 0.0 {
            return None;
        }

        let remaining_tokens = token_limit - current_tokens;
        let minutes_to_limit = remaining_tokens as f64 / current_burn_rate;

        Some(Duration::minutes(minutes_to_limit as i64))
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::UsageEntry;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_burn_rate() {
        let calculator = Calculator::new();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end_time = start_time + Duration::hours(1);

        let mut block = SessionBlock::new(start_time, end_time);
        let entry = UsageEntry::new(
            start_time,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );
        block.add_entry(entry);

        let burn_rate = calculator.calculate_burn_rate(&block).unwrap();
        assert_eq!(burn_rate.tokens_per_minute(), 150.0);
        assert_eq!(burn_rate.cost_per_hour(), 0.06);
    }

    #[test]
    fn test_empty_block_returns_none() {
        let calculator = Calculator::new();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end_time = start_time + Duration::hours(1);

        let block = SessionBlock::new(start_time, end_time);
        assert!(calculator.calculate_burn_rate(&block).is_none());
    }

    #[test]
    fn test_project_block_usage() {
        let calculator = Calculator::new();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let end_time = start_time + Duration::hours(2);
        let current_time = start_time + Duration::hours(1);

        let mut block = SessionBlock::new(start_time, end_time);
        let entry = UsageEntry::new(
            start_time,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );
        block.add_entry(entry);

        let projection = calculator
            .project_block_usage(&block, current_time)
            .unwrap();
        assert_eq!(projection.current_tokens(), 150);
        assert_eq!(projection.current_cost(), 0.001);
        assert!(projection.projected_additional_tokens() > 0);
        assert!(projection.projected_additional_cost() > 0.0);
    }

    #[test]
    fn test_calculate_hourly_burn_rate() {
        let calculator = Calculator::new();
        let current_time = Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap();
        let start_time = current_time - Duration::minutes(30);
        let end_time = current_time - Duration::minutes(15);

        let mut block = SessionBlock::new(start_time, current_time);
        let entry1 = UsageEntry::new(
            start_time,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );
        let entry2 = UsageEntry::new(
            end_time,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );
        block.add_entry(entry1);
        block.add_entry(entry2);

        let hourly_rate = calculator.calculate_hourly_burn_rate(&[block], current_time);
        assert!(hourly_rate > 0.0);
    }

    #[test]
    fn test_calculate_time_to_limit() {
        let calculator = Calculator::new();

        let time_to_limit = calculator.calculate_time_to_limit(500, 1000, 5.0).unwrap();
        assert_eq!(time_to_limit, Duration::minutes(100));
    }

    #[test]
    fn test_time_to_limit_already_exceeded() {
        let calculator = Calculator::new();

        let time_to_limit = calculator.calculate_time_to_limit(1000, 500, 5.0);
        assert!(time_to_limit.is_none());
    }
}
