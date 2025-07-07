use crate::data_structures::ModelPricing;
use std::collections::HashMap;

pub struct PricingProvider {
    pricing_cache: HashMap<String, ModelPricing>,
}

impl PricingProvider {
    pub fn new() -> Self {
        let mut pricing_cache = HashMap::new();

        pricing_cache.insert(
            "claude-3-opus-20240229".to_string(),
            ModelPricing::new(
                15.0 / 1_000_000.0,  // $15 per 1M input tokens
                75.0 / 1_000_000.0,  // $75 per 1M output tokens
                18.75 / 1_000_000.0, // $18.75 per 1M cache creation tokens
                1.875 / 1_000_000.0, // $1.875 per 1M cache read tokens
            ),
        );

        pricing_cache.insert(
            "claude-3-sonnet-20240229".to_string(),
            ModelPricing::new(
                3.0 / 1_000_000.0,  // $3 per 1M input tokens
                15.0 / 1_000_000.0, // $15 per 1M output tokens
                3.75 / 1_000_000.0, // $3.75 per 1M cache creation tokens
                0.3 / 1_000_000.0,  // $0.3 per 1M cache read tokens
            ),
        );

        pricing_cache.insert(
            "claude-3-haiku-20240307".to_string(),
            ModelPricing::new(
                0.25 / 1_000_000.0, // $0.25 per 1M input tokens
                1.25 / 1_000_000.0, // $1.25 per 1M output tokens
                0.3 / 1_000_000.0,  // $0.3 per 1M cache creation tokens
                0.03 / 1_000_000.0, // $0.03 per 1M cache read tokens
            ),
        );

        pricing_cache.insert(
            "claude-3-5-sonnet-20240620".to_string(),
            ModelPricing::new(
                3.0 / 1_000_000.0,  // $3 per 1M input tokens
                15.0 / 1_000_000.0, // $15 per 1M output tokens
                3.75 / 1_000_000.0, // $3.75 per 1M cache creation tokens
                0.3 / 1_000_000.0,  // $0.3 per 1M cache read tokens
            ),
        );

        pricing_cache.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            ModelPricing::new(
                3.0 / 1_000_000.0,  // $3 per 1M input tokens
                15.0 / 1_000_000.0, // $15 per 1M output tokens
                3.75 / 1_000_000.0, // $3.75 per 1M cache creation tokens
                0.3 / 1_000_000.0,  // $0.3 per 1M cache read tokens
            ),
        );

        pricing_cache.insert(
            "claude-3-5-haiku-20241022".to_string(),
            ModelPricing::new(
                1.0 / 1_000_000.0,  // $1 per 1M input tokens
                5.0 / 1_000_000.0,  // $5 per 1M output tokens
                1.25 / 1_000_000.0, // $1.25 per 1M cache creation tokens
                0.1 / 1_000_000.0,  // $0.1 per 1M cache read tokens
            ),
        );

        // Add support for newer model names
        pricing_cache.insert(
            "claude-opus-4-20250514".to_string(),
            ModelPricing::new(
                15.0 / 1_000_000.0,  // $15 per 1M input tokens
                75.0 / 1_000_000.0,  // $75 per 1M output tokens
                18.75 / 1_000_000.0, // $18.75 per 1M cache creation tokens
                1.875 / 1_000_000.0, // $1.875 per 1M cache read tokens
            ),
        );

        pricing_cache.insert(
            "claude-sonnet-4-20250514".to_string(),
            ModelPricing::new(
                3.0 / 1_000_000.0,  // $3 per 1M input tokens
                15.0 / 1_000_000.0, // $15 per 1M output tokens
                3.75 / 1_000_000.0, // $3.75 per 1M cache creation tokens
                0.3 / 1_000_000.0,  // $0.3 per 1M cache read tokens
            ),
        );

        Self { pricing_cache }
    }

    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.pricing_cache.get(model)
    }

    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
    ) -> Option<f64> {
        self.pricing_cache.get(model).map(|pricing| {
            pricing.calculate_cost(
                input_tokens,
                output_tokens,
                cache_creation_tokens,
                cache_read_tokens,
            )
        })
    }

    pub fn get_model_weight(&self, model: &str) -> f64 {
        match model {
            "claude-3-opus-20240229" | "claude-opus-4-20250514" => 5.0,
            "claude-3-sonnet-20240229"
            | "claude-3-5-sonnet-20240620"
            | "claude-3-5-sonnet-20241022"
            | "claude-sonnet-4-20250514" => 1.0,
            "claude-3-haiku-20240307" | "claude-3-5-haiku-20241022" => 0.2,
            _ => 1.0,
        }
    }

    pub fn supported_models(&self) -> Vec<&String> {
        self.pricing_cache.keys().collect()
    }
}

impl Default for PricingProvider {
    fn default() -> Self {
        Self::new()
    }
}
