use crate::data_structures::UsageEntry;
use crate::pricing::PricingProvider;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataLoader {
    pricing_provider: PricingProvider,
}

impl DataLoader {
    pub fn new() -> Self {
        Self {
            pricing_provider: PricingProvider::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<UsageEntry>> {
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.as_ref().display()))?;
        
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.with_context(|| format!("Failed to read line {}", line_num + 1))?;
            
            if line.trim().is_empty() {
                continue;
            }

            match self.parse_line(&line) {
                Ok(entry) => entries.push(entry),
                Err(_) => {
                    // Silently skip lines that don't contain usage data
                    continue;
                }
            }
        }

        Ok(entries)
    }

    pub fn load_from_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<Vec<UsageEntry>> {
        let mut all_entries = Vec::new();
        self.load_from_directory_recursive(dir_path.as_ref(), &mut all_entries)?;
        all_entries.sort_by(|a, b| a.timestamp().cmp(&b.timestamp()));
        Ok(all_entries)
    }

    fn load_from_directory_recursive(&self, dir_path: &Path, entries: &mut Vec<UsageEntry>) -> Result<()> {
        let dir = std::fs::read_dir(dir_path)
            .with_context(|| format!("Failed to read directory: {}", dir_path.display()))?;

        for entry in dir {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "jsonl" {
                        match self.load_from_file(&path) {
                            Ok(mut file_entries) => entries.append(&mut file_entries),
                            Err(e) => {
                                eprintln!("Warning: Failed to load file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                // Recursively search subdirectories (for project directories)
                if let Err(e) = self.load_from_directory_recursive(&path, entries) {
                    eprintln!("Warning: Failed to load from directory {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<UsageEntry> {
        let json: Value = serde_json::from_str(line)
            .context("Failed to parse JSON")?;

        // Check if this is an assistant message with usage data
        if let Some(message) = json.get("message") {
            if let Some(usage) = message.get("usage") {
                let timestamp = self.parse_timestamp(&json)?;
                let model = message.get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let input_tokens = self.extract_u64(usage, "input_tokens")?;
                let output_tokens = self.extract_u64(usage, "output_tokens")?;
                let cache_creation_input_tokens = self.extract_u64(usage, "cache_creation_input_tokens").unwrap_or(0);
                let cache_read_input_tokens = self.extract_u64(usage, "cache_read_input_tokens").unwrap_or(0);

                // Calculate cost using our pricing (since cost_usd might not be present)
                let cost_usd = self.extract_f64(&json, "cost_usd").unwrap_or_else(|_| {
                    self.pricing_provider.calculate_cost(
                        &model,
                        input_tokens,
                        output_tokens,
                        cache_creation_input_tokens,
                        cache_read_input_tokens,
                    ).unwrap_or(0.0)
                });

                return Ok(UsageEntry::new(
                    timestamp,
                    model,
                    input_tokens,
                    output_tokens,
                    cache_creation_input_tokens,
                    cache_read_input_tokens,
                    cost_usd,
                ));
            }
        }

        // Fallback: try to parse as the simple format used in tests
        if let Some(usage) = json.get("usage") {
            let timestamp = self.parse_timestamp(&json)?;
            let model = self.extract_string(&json, "model")?;

            let input_tokens = self.extract_u64(usage, "input_tokens")?;
            let output_tokens = self.extract_u64(usage, "output_tokens")?;
            let cache_creation_input_tokens = self.extract_u64(usage, "cache_creation_input_tokens").unwrap_or(0);
            let cache_read_input_tokens = self.extract_u64(usage, "cache_read_input_tokens").unwrap_or(0);

            let cost_usd = self.extract_f64(&json, "cost_usd").unwrap_or(0.0);

            Ok(UsageEntry::new(
                timestamp,
                model,
                input_tokens,
                output_tokens,
                cache_creation_input_tokens,
                cache_read_input_tokens,
                cost_usd,
            ))
        } else {
            Err(anyhow::anyhow!("No usage data found in this entry"))
        }
    }

    fn parse_timestamp(&self, json: &Value) -> Result<DateTime<Utc>> {
        let timestamp_str = self.extract_string(json, "timestamp")?;
        
        DateTime::parse_from_rfc3339(&timestamp_str)
            .context("Failed to parse timestamp")
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn extract_string(&self, json: &Value, key: &str) -> Result<String> {
        json.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid '{}' field", key))
    }

    fn extract_u64(&self, json: &Value, key: &str) -> Result<u64> {
        json.get(key)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid '{}' field", key))
    }

    fn extract_f64(&self, json: &Value, key: &str) -> Result<f64> {
        json.get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid '{}' field", key))
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_line() {
        let loader = DataLoader::new();
        let line = r#"{"timestamp": "2024-01-01T12:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 100, "output_tokens": 50, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}, "cost_usd": 0.001}"#;
        
        let entry = loader.parse_line(line).unwrap();
        assert_eq!(entry.model(), "claude-3-sonnet-20240229");
        assert_eq!(entry.input_tokens(), 100);
        assert_eq!(entry.output_tokens(), 50);
        assert_eq!(entry.cost_usd(), 0.001);
    }

    #[test]
    fn test_parse_line_with_cache_tokens() {
        let loader = DataLoader::new();
        let line = r#"{"timestamp": "2024-01-01T12:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 100, "output_tokens": 50, "cache_creation_input_tokens": 25, "cache_read_input_tokens": 10}, "cost_usd": 0.001}"#;
        
        let entry = loader.parse_line(line).unwrap();
        assert_eq!(entry.cache_creation_input_tokens(), 25);
        assert_eq!(entry.cache_read_input_tokens(), 10);
    }

    #[test]
    fn test_load_from_file() {
        let loader = DataLoader::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let content = r#"{"timestamp": "2024-01-01T12:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 100, "output_tokens": 50, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}, "cost_usd": 0.001}
{"timestamp": "2024-01-01T13:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 200, "output_tokens": 100, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}, "cost_usd": 0.002}"#;
        
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let entries = loader.load_from_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].input_tokens(), 100);
        assert_eq!(entries[1].input_tokens(), 200);
    }

    #[test]
    fn test_load_from_file_with_empty_lines() {
        let loader = DataLoader::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let content = r#"{"timestamp": "2024-01-01T12:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 100, "output_tokens": 50, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}, "cost_usd": 0.001}

{"timestamp": "2024-01-01T13:00:00Z", "model": "claude-3-sonnet-20240229", "usage": {"input_tokens": 200, "output_tokens": 100, "cache_creation_input_tokens": 0, "cache_read_input_tokens": 0}, "cost_usd": 0.002}"#;
        
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let entries = loader.load_from_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_invalid_json_line() {
        let loader = DataLoader::new();
        let line = r#"{"invalid": "json"#;
        
        assert!(loader.parse_line(line).is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let loader = DataLoader::new();
        let line = r#"{"timestamp": "2024-01-01T12:00:00Z"}"#;
        
        assert!(loader.parse_line(line).is_err());
    }
}