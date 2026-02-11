use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::env;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub starting_balance: Decimal,
    pub min_scan_interval_seconds: u64,
    pub max_scan_interval_seconds: u64,
    pub claude_api_key: String,
    pub claude_model: String,
    pub claude_api_cost_per_call: Decimal,
    pub news_api_key: String,
    pub polygon_api_key: String,
    pub finnhub_api_key: String,
    pub max_position_percent: Decimal,
    pub min_mispricing_percent: Decimal,
    pub vps_cost_per_month: Decimal,
    pub simulate_markets: bool,
}

impl Settings {
    pub fn new() -> Result<Self> {
        dotenv::dotenv().ok();
        
        // Use environment variables or defaults for testing
        let claude_api_key = env::var("CLAUDE_API_KEY")
            .unwrap_or_else(|_| "test_key".to_string());
        
        let news_api_key = env::var("NEWS_API_KEY")
            .unwrap_or_else(|_| "test_news_key".to_string());

        let claude_model = env::var("CLAUDE_MODEL")
            .unwrap_or_else(|_| "claude-3-haiku-20240307".to_string());
        
        Ok(Self {
            starting_balance: dec!(50.00), // $50 starting capital
            min_scan_interval_seconds: 60, // 1 minute
            max_scan_interval_seconds: 300, // 5 minutes
            claude_api_key,
            claude_model,
            claude_api_cost_per_call: dec!(0.01), // $0.01 per API call
            news_api_key,
            polygon_api_key: env::var("POLYGON_API_KEY")
                .unwrap_or_else(|_| "test_polygon_key".to_string()),
            finnhub_api_key: env::var("FINNHUB_API_KEY")
                .unwrap_or_else(|_| "test_finnhub_key".to_string()),
            max_position_percent: dec!(0.06), // Max 6% of bankroll
            min_mispricing_percent: dec!(8.0), // Minimum 8% mispricing
            vps_cost_per_month: dec!(4.5), // $4.5/month VPS
            simulate_markets: true, // For testing without real API access
        })
    }
    
    pub fn is_test_mode(&self) -> bool {
        self.claude_api_key == "test_key" || self.simulate_markets
    }
}
