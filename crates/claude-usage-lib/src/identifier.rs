use crate::data_structures::{SessionBlock, UsageEntry};
use chrono::{DateTime, Duration, Timelike, Utc};

pub struct SessionIdentifier {
    session_duration: Duration,
}

impl SessionIdentifier {
    pub fn new() -> Self {
        Self {
            session_duration: Duration::hours(5),
        }
    }

    pub fn with_duration(session_duration: Duration) -> Self {
        Self { session_duration }
    }

    pub fn identify_blocks(&self, entries: &[UsageEntry]) -> Vec<SessionBlock> {
        if entries.is_empty() {
            return Vec::new();
        }

        let mut blocks = Vec::new();
        let mut current_block = None;

        for entry in entries {
            if let Some(ref mut block) = current_block {
                if self.should_create_new_block(block, entry) {
                    blocks.push(block.clone());
                    current_block = Some(self.create_block_for_entry(entry));
                    current_block.as_mut().unwrap().add_entry(entry.clone());
                } else {
                    block.add_entry(entry.clone());
                }
            } else {
                current_block = Some(self.create_block_for_entry(entry));
                current_block.as_mut().unwrap().add_entry(entry.clone());
            }
        }

        if let Some(block) = current_block {
            blocks.push(block);
        }

        blocks
    }

    fn should_create_new_block(&self, block: &SessionBlock, entry: &UsageEntry) -> bool {
        if entry.timestamp() >= block.end_time() {
            return true;
        }

        if let Some(last_entry) = block.entries().last() {
            if entry.timestamp() - last_entry.timestamp() >= self.session_duration {
                return true;
            }
        }

        false
    }

    fn create_block_for_entry(&self, entry: &UsageEntry) -> SessionBlock {
        let start_time = self.round_to_hour(entry.timestamp());
        let end_time = start_time + self.session_duration;
        
        SessionBlock::new(start_time, end_time)
    }

    fn round_to_hour(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        timestamp
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }
}

impl Default for SessionIdentifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_single_entry() {
        let identifier = SessionIdentifier::new();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 30, 0).unwrap();
        let entry = UsageEntry::new(
            timestamp,
            "claude-3-sonnet-20240229".to_string(),
            100,
            50,
            0,
            0,
            0.001,
        );

        let blocks = identifier.identify_blocks(&[entry]);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].entries().len(), 1);
    }

    #[test]
    fn test_multiple_entries_same_block() {
        let identifier = SessionIdentifier::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        
        let entries = vec![
            UsageEntry::new(base_time, "claude-3-sonnet-20240229".to_string(), 100, 50, 0, 0, 0.001),
            UsageEntry::new(base_time + Duration::hours(1), "claude-3-sonnet-20240229".to_string(), 150, 75, 0, 0, 0.002),
            UsageEntry::new(base_time + Duration::hours(2), "claude-3-sonnet-20240229".to_string(), 200, 100, 0, 0, 0.003),
        ];

        let blocks = identifier.identify_blocks(&entries);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].entries().len(), 3);
    }

    #[test]
    fn test_gap_creates_new_block() {
        let identifier = SessionIdentifier::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        
        let entries = vec![
            UsageEntry::new(base_time, "claude-3-sonnet-20240229".to_string(), 100, 50, 0, 0, 0.001),
            UsageEntry::new(base_time + Duration::hours(6), "claude-3-sonnet-20240229".to_string(), 150, 75, 0, 0, 0.002),
        ];

        let blocks = identifier.identify_blocks(&entries);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].entries().len(), 1);
        assert_eq!(blocks[1].entries().len(), 1);
    }

    #[test]
    fn test_time_boundary_creates_new_block() {
        let identifier = SessionIdentifier::new();
        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        
        let entries = vec![
            UsageEntry::new(base_time, "claude-3-sonnet-20240229".to_string(), 100, 50, 0, 0, 0.001),
            UsageEntry::new(base_time + Duration::hours(5), "claude-3-sonnet-20240229".to_string(), 150, 75, 0, 0, 0.002),
        ];

        let blocks = identifier.identify_blocks(&entries);
        assert_eq!(blocks.len(), 2);
    }
}