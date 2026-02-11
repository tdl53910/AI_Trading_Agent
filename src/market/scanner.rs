use crate::agent::Market;
use crate::config::Settings;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use chrono::{Utc, Duration};
use anyhow::Result;
use rand::Rng;
use log::info;

#[derive(Debug, Clone)]
pub struct MarketScanner;

impl MarketScanner {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn scan_markets(
        &self,
        count: usize,
        simulate_markets: bool,
        settings: &Settings,
    ) -> Result<Vec<Market>> {
        info!("🎯 Scanning {} markets...", count);
        
        let mut markets = Vec::with_capacity(count);
        
        for i in 0..count {
            if simulate_markets {
                let market = self.generate_simulated_market(i, settings);
                markets.push(market);
            } else {
                let market = self.get_real_market_data(&format!("market_{}", i)).await?;
                markets.push(market);
            }
        }
        
        info!(
            "✅ Generated {} {} markets",
            markets.len(),
            if simulate_markets { "simulated" } else { "real" }
        );
        Ok(markets)
    }
    
    fn generate_simulated_market(&self, id: usize, _settings: &Settings) -> Market {
        let mut rng = rand::thread_rng();
        
        let categories = [
            "Politics", "Sports", "Finance", "Crypto", "Technology",
            "Entertainment", "Science", "Weather", "Economics", "Stocks"
        ];
        
        let category = categories[id % categories.len()];
        
        let price = dec!(10) + Decimal::from(rng.gen_range(0..90));
        
        let market_names = [
            "Will Bitcoin reach $100K by 2025?",
            "Election outcome prediction",
            "Sports championship winner",
            "Company earnings report",
            "Federal reserve rate decision",
            "Movie box office performance",
            "Weather event probability",
            "Product launch success",
            "Economic indicator forecast",
            "Celebrity event outcome",
        ];
        
        let name = format!("{} #{}", market_names[id % market_names.len()], id);
        
        let mut history = Vec::new();
        let mut hist_price = price;
        for _ in 0..100 {
            let change: f64 = rng.gen_range(-0.05..0.05);
            let change_dec = Decimal::try_from(change).unwrap_or(dec!(0));
            hist_price = hist_price * (dec!(1) + change_dec);
            hist_price = hist_price.max(dec!(0.01));
            history.push(hist_price);
        }
        
        let current_price = *history.last().unwrap_or(&price);
        
        let expiry_date = if rng.gen_bool(0.7) {
            Some(Utc::now() + Duration::days(rng.gen_range(1..365)))
        } else {
            None
        };
        
        Market {
            id: format!("market_{}", id),
            name,
            description: format!("Simulated {} market for testing AI agent", category),
            current_price,
            volume: Decimal::from(rng.gen_range(1000..1000000)),
            liquidity: Decimal::from(rng.gen_range(10000..1000000)),
            history,
            category: category.to_string(),
            expiry_date,
        }
    }
    
    pub async fn get_real_market_data(&self, _market_id: &str) -> Result<Market> {
        Ok(Market {
            id: "real_market".to_string(),
            name: "Real Market Placeholder".to_string(),
            description: "Real market data would go here".to_string(),
            current_price: dec!(50),
            volume: dec!(100000),
            liquidity: dec!(500000),
            history: vec![dec!(45), dec!(48), dec!(52), dec!(50)],
            category: "Finance".to_string(),
            expiry_date: Some(Utc::now() + Duration::days(30)),
        })
    }
}
