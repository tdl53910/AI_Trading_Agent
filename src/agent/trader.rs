use crate::market::scanner::MarketScanner;
use crate::news::analyzer::{NewsAnalyzer};
use crate::config::Settings;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::{Result, Context};
use log::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub market_id: String,
    pub market_name: String,
    pub position: Position,
    pub amount: Decimal,
    pub entry_price: Decimal,
    pub exit_price: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
    pub profit_loss: Option<Decimal>,
    pub status: TradeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Position {
    Long,
    Short,
    Yes,
    No,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeStatus {
    Open,
    Closed,
    Stopped,
}

pub struct Trader {
    pub balance: Decimal,
    pub portfolio: HashMap<String, Trade>,
    pub trade_history: VecDeque<Trade>,
    pub total_profit: Decimal,
    pub total_loss: Decimal,
    pub trades_executed: usize,
    pub is_test_mode: bool,
    pub simulate_markets: bool,
    pub is_paused: bool,
    pub preferred_categories: Vec<String>,
    pub decision_log: VecDeque<String>,
    pub is_alive: bool,
    pub news_sources: Vec<String>,
    pub last_news: Vec<crate::news::analyzer::NewsArticle>,
    pub last_instruction: Option<String>,
    pub last_llm_response: Option<String>,
    pub news_analyzer: NewsAnalyzer,
    pub market_scanner: MarketScanner,
    pub cycle_count: u32,
    pub last_news_update: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Market {
    pub id: String,
    pub name: String,
    pub description: String,
    pub current_price: Decimal,
    pub volume: Decimal,
    pub liquidity: Decimal,
    pub history: Vec<Decimal>,
    pub category: String,
    pub expiry_date: Option<DateTime<Utc>>,
}

impl Trader {
    pub fn new(starting_balance: Decimal) -> Self {
        let news_analyzer = NewsAnalyzer::new("test_key".to_string());
        
        Self {
            balance: starting_balance,
            portfolio: HashMap::new(),
            trade_history: VecDeque::with_capacity(1000),
            total_profit: dec!(0),
            total_loss: dec!(0),
            trades_executed: 0,
            is_test_mode: true, // Start in test mode for safety
            simulate_markets: true,
            is_paused: false,
            preferred_categories: Vec::new(),
            decision_log: VecDeque::with_capacity(200),
            is_alive: true,
            news_sources: vec![
                "https://newsapi.org/v2/everything?q=finance&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=stock+market&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=crypto+currency&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=economy&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=technology&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=energy&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=markets&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=inflation&language=en".to_string(),
                "https://newsapi.org/v2/everything?q=interest+rates&language=en".to_string(),
            ],
            last_news: Vec::new(),
            last_instruction: None,
            last_llm_response: None,
            news_analyzer,
            market_scanner: MarketScanner::new(),
            cycle_count: 0,
            last_news_update: None,
        }
    }

    pub fn from_settings(settings: &Settings) -> Self {
        let mut trader = Trader::new(settings.starting_balance);
        trader.is_test_mode = settings.is_test_mode();
        trader.simulate_markets = settings.simulate_markets;
        trader.news_analyzer = NewsAnalyzer::new(settings.news_api_key.clone());
        trader
    }
    
    pub fn is_alive(&self) -> bool {
        self.is_alive && self.balance > dec!(0)
    }
    
    pub async fn scan_markets(&mut self, settings: &Settings) -> Result<()> {
        self.cycle_count += 1;
        
        // Scan markets (500-1000 as described)
        let market_count = 500 + (rand::random::<usize>() % 501); // Random between 500-1000
        info!("🔍 Scanning {} markets...", market_count);
        
        let markets = self.market_scanner
            .scan_markets(market_count, self.simulate_markets, settings).await
            .context("Failed to scan markets")?;
        
        info!("📊 Found {} markets to analyze", markets.len());
        
        // Analyze each market
        let mut opportunities = Vec::new();
        for (i, market) in markets.iter().enumerate() {
            if i % 100 == 0 {
                info!("📈 Analyzed {}/{} markets", i, markets.len());
            }
            
            // Estimate fair value (simulated for now)
            let fair_value = self.estimate_fair_value(market).await
                .context("Failed to estimate fair value")?;
            
            // Calculate mispricing percentage
            let price_diff = if market.current_price > fair_value {
                market.current_price - fair_value
            } else {
                fair_value - market.current_price
            };
            
            let mispricing = (price_diff / fair_value) * dec!(100);
            
            if mispricing > settings.min_mispricing_percent {
                opportunities.push((market, fair_value, mispricing));
                info!("🎯 Found opportunity: {} (Mispricing: {}%)", 
                      market.name, mispricing.round_dp(2));
            }
        }
        
        info!("🎯 Found {} trading opportunities", opportunities.len());
        
        // Store opportunities for execution
        self.evaluate_opportunities(opportunities, settings).await?;
        
        Ok(())
    }
    
    async fn estimate_fair_value(&self, market: &Market) -> Result<Decimal> {
        // In a real implementation, this would call Claude API
        // For simulation, we add some randomness based on news sentiment
        
        let mut base_value = market.current_price;
        
        // Simulate news impact
        let news_impact = if rand::random::<f64>() > 0.7 {
            dec!(0.05) // 5% positive
        } else if rand::random::<f64>() < 0.3 {
            dec!(-0.03) // -3% negative
        } else {
            dec!(0) // Neutral
        };
        
        // Add some mean reversion tendency
        let mean_price: Decimal = market.history.iter().sum::<Decimal>() / Decimal::from(market.history.len());
        let mean_reversion = (mean_price - market.current_price) * dec!(0.1);
        
        // Technical indicators (simplified)
        let rsi_effect = if market.current_price > market.history.iter().rev().take(14).sum::<Decimal>() / dec!(14) {
            dec!(-0.02) // Overbought
        } else {
            dec!(0.01) // Oversold
        };
        
        let fair_value = market.current_price * (dec!(1) + news_impact + mean_reversion / market.current_price + rsi_effect);
        
        // Add some randomness for simulation
        let random_factor = dec!(0);
        
        Ok(fair_value * (dec!(1) + random_factor))
    }
    
    async fn evaluate_opportunities(&mut self, opportunities: Vec<(&Market, Decimal, Decimal)>, settings: &Settings) -> Result<()> {
        for (market, fair_value, mispricing) in opportunities {
            if !self.preferred_categories.is_empty()
                && !self.preferred_categories.iter().any(|cat| cat.eq_ignore_ascii_case(&market.category))
            {
                self.push_decision(format!(
                    "Skipped {}: category {} not in preferences",
                    market.name, market.category
                ));
                continue;
            }
            // Calculate position size using Kelly Criterion
            let kelly_fraction = self.calculate_kelly_criterion(market, fair_value, mispricing);
            let max_position = self.balance * settings.max_position_percent;
            let position_size = (self.balance * kelly_fraction).min(max_position).max(dec!(1));
            
            if position_size < dec!(5) {
                self.push_decision(format!(
                    "Skipped {}: position size ${} too small",
                    market.name, position_size.round_dp(2)
                ));
                continue; // Position too small
            }
            
            // Check if we have enough balance
            if position_size > self.balance * dec!(0.9) {
                warn!("⚠️  Insufficient balance for position: ${} (Balance: ${})", 
                      position_size, self.balance);
                self.push_decision(format!(
                    "Skipped {}: insufficient balance for ${}",
                    market.name, position_size.round_dp(2)
                ));
                continue;
            }
            
            // Determine trade direction
            let position = if market.current_price < fair_value {
                Position::Long
            } else {
                Position::Short
            };
            
            info!("🎮 Executing trade: {:?} {} @ ${} (Size: ${})", 
                  position, market.name, market.current_price, position_size);
            self.push_decision(format!(
                "Trade {:?} {} @ ${} (size ${})",
                position,
                market.name,
                market.current_price.round_dp(2),
                position_size.round_dp(2)
            ));
            
            // Execute trade
            self.execute_trade(market.clone(), position, position_size, market.current_price).await?;
        }
        
        Ok(())
    }
    
    pub fn calculate_kelly_criterion(&self, market: &Market, fair_value: Decimal, mispricing: Decimal) -> Decimal {
        // Simplified Kelly Criterion
        // f* = p - q/b
        // where:
        // p = probability of winning
        // q = probability of losing (1 - p)
        // b = net odds received on the bet
        
        // Base probability on mispricing magnitude
        let base_prob = (mispricing.min(dec!(20)) / dec!(20)) * dec!(0.8) + dec!(0.2); // 20-100% probability
        
        // Adjust based on market liquidity
        let liquidity_factor = if market.liquidity > dec!(100000) {
            dec!(1.1)
        } else if market.liquidity > dec!(10000) {
            dec!(1.0)
        } else {
            dec!(0.9)
        };
        
        let p = base_prob * liquidity_factor;
        let q = dec!(1) - p;
        let b = dec!(2); // Assuming 2:1 odds for simplicity
        
        let kelly = (p - (q / b)).max(dec!(0)).min(dec!(0.25)); // Cap at 25%
        
        info!("🧮 Kelly Criterion: p={:.2}, q={:.2}, b={}, f*={:.2}%", 
              p, q, b, kelly * dec!(100));
        
        kelly
    }
    
    pub async fn execute_trade(
        &mut self,
        market: Market,
        position: Position,
        amount: Decimal,
        price: Decimal,
    ) -> Result<Decimal> {
        let trade_id = Uuid::new_v4().to_string();
        
        // Calculate expected profit (simulated)
        let expected_return = if rand::random::<f64>() > 0.6 {
            // 60% chance of profit
            amount * dec!(0.15) // 15% profit
        } else if rand::random::<f64>() < 0.3 {
            // 30% chance of small loss
            amount * dec!(-0.05) // -5% loss
        } else {
            // 10% chance of larger loss
            amount * dec!(-0.12) // -12% loss
        };
        
        // In test mode, we simulate the trade
        let trade = Trade {
            id: trade_id.clone(),
            market_id: market.id.clone(),
            market_name: market.name.clone(),
            position: position.clone(),
            amount,
            entry_price: price,
            exit_price: Some(price * (dec!(1) + expected_return / amount)),
            timestamp: Utc::now(),
            profit_loss: Some(expected_return),
            status: TradeStatus::Closed,
        };
        
        // Update portfolio
        self.portfolio.insert(trade_id.clone(), trade.clone());
        self.trade_history.push_back(trade.clone());
        
        // Update balance and stats
        self.update_balance(expected_return);
        
        self.trades_executed += 1;
        
        if expected_return > dec!(0) {
            info!("💰 Trade PROFIT: +${}", expected_return);
        } else {
            info!("💸 Trade LOSS: ${}", expected_return);
        }
        
        Ok(expected_return)
    }
    
    pub fn update_balance(&mut self, profit_loss: Decimal) {
        self.balance += profit_loss;
        
        if profit_loss > dec!(0) {
            self.total_profit += profit_loss;
        } else {
            self.total_loss += profit_loss.abs();
        }
        
        // Check if agent died
        if self.balance <= dec!(0) {
            self.is_alive = false;
            error!("💀 AGENT DIED! Balance: ${}", self.balance);
        }
        
        // Update is_test_mode based on balance (simulate real mode after success)
        if self.balance > dec!(1000) && self.is_test_mode {
            info!("🚀 Switching to REAL mode! Balance exceeded $1000");
            self.is_test_mode = false;
        }
    }
    
    pub fn pay_api_bill(&mut self, settings: &Settings) {
        // Simulate API costs
        let api_calls = self.trades_executed + self.cycle_count as usize;
        let api_cost = settings.claude_api_cost_per_call * Decimal::from(api_calls);
        
        if self.balance >= api_cost * dec!(2) {
            self.balance -= api_cost;
            info!("💳 Paid API bill: ${}", api_cost);
        } else {
            warn!("⚠️  Low balance, skipping API payment");
        }
        
        // Pay VPS cost monthly (simulated)
        if self.cycle_count % (30 * 24 * 6) == 0 { // Every ~30 days (6 cycles/hour * 24 hours * 30 days)
            let vps_daily_cost = settings.vps_cost_per_month / dec!(30);
            if self.balance >= vps_daily_cost {
                self.balance -= vps_daily_cost;
                info!("🏠 Paid VPS cost: ${}", vps_daily_cost);
            }
        }
    }
    
    pub async fn update_news_analysis(&mut self, settings: &Settings) -> Result<()> {
        info!("📰 Updating news analysis...");
        
        let now = Utc::now();
        self.last_news_update = Some(now);
        
        // Fetch news from all sources
        let mut all_articles = Vec::new();
        for source in &self.news_sources {
            match self.news_analyzer.fetch_news(source, "markets").await {
                Ok(articles) => {
                    info!("📖 Found {} articles from {}", articles.len(), source);
                    all_articles.extend(articles);
                },
                Err(e) => {
                    warn!("⚠️  Failed to fetch news from {}: {}", source, e);
                }
            }
        }
        
        // Analyze sentiment
        for article in &mut all_articles {
            article.sentiment = self.news_analyzer.analyze_sentiment(&article.title).await
                .unwrap_or(0.0);
        }

        self.last_news = all_articles.clone();
        
        info!("🧠 Analyzed {} news articles", all_articles.len());
        
        Ok(())
    }
    
    pub fn log_status(&self) {
        let net_profit = self.total_profit - self.total_loss;
        let win_rate = if self.trades_executed > 0 {
            let profitable_trades = self.trade_history.iter()
                .filter(|t| t.profit_loss.unwrap_or(dec!(0)) > dec!(0))
                .count();
            (profitable_trades as f64 / self.trades_executed as f64) * 100.0
        } else {
            0.0
        };
        
        info!("📊 ========== AGENT STATUS ==========");
        info!("💰 Balance: ${}", self.balance.round_dp(2));
        info!("📈 Net Profit: ${}", net_profit.round_dp(2));
        info!("✅ Total Profit: ${}", self.total_profit.round_dp(2));
        info!("❌ Total Loss: ${}", self.total_loss.round_dp(2));
        info!("🎯 Trades Executed: {}", self.trades_executed);
        info!("📊 Win Rate: {:.1}%", win_rate);
        info!("🔧 Mode: {}", if self.is_test_mode { "TEST" } else { "REAL" });
        info!("❤️  Status: {}", if self.is_alive { "ALIVE" } else { "DEAD" });
        info!("🔄 Cycles Completed: {}", self.cycle_count);
        info!("=====================================");
    }
    
    pub fn toggle_test_mode(&mut self) {
        self.is_test_mode = !self.is_test_mode;
        info!("🔄 Test mode: {}", if self.is_test_mode { "ON" } else { "OFF" });
    }

    pub fn toggle_simulation(&mut self) {
        self.simulate_markets = !self.simulate_markets;
        self.is_test_mode = self.simulate_markets;
        info!("🧪 Simulated markets: {}", if self.simulate_markets { "ON" } else { "OFF" });
    }

    pub fn toggle_running(&mut self) {
        self.is_paused = !self.is_paused;
        info!("⏯️  Trading: {}", if self.is_paused { "PAUSED" } else { "RUNNING" });
    }

    pub fn apply_instruction(&mut self, instruction: &str) -> Vec<String> {
        let text = instruction.to_lowercase();

        if text.contains("all") || text.contains("any") || text.contains("everything") {
            self.preferred_categories.clear();
            self.push_decision("Cleared category filter (all markets)".to_string());
            return self.preferred_categories.clone();
        }

        let mut categories = Vec::new();

        if text.contains("crypto") { categories.push("Crypto"); }
        if text.contains("stock") { categories.push("Stocks"); }
        if text.contains("tech") { categories.push("Technology"); }
        if text.contains("finance") { categories.push("Finance"); }
        if text.contains("econom") { categories.push("Economics"); }
        if text.contains("politic") { categories.push("Politics"); }
        if text.contains("sport") { categories.push("Sports"); }
        if text.contains("weather") { categories.push("Weather"); }
        if text.contains("entertain") { categories.push("Entertainment"); }
        if text.contains("science") { categories.push("Science"); }

        self.preferred_categories = categories.iter().map(|c| c.to_string()).collect();
        if self.preferred_categories.is_empty() {
            self.push_decision("No recognized categories in instruction".to_string());
        } else {
            self.push_decision(format!(
                "Set preferred categories: {}",
                self.preferred_categories.join(", ")
            ));
        }
        self.preferred_categories.clone()
    }

    pub async fn simulate_day(&mut self, settings: &Settings) -> Result<usize> {
        if !self.is_test_mode || !self.simulate_markets {
            self.push_decision("Simulation requested but not in test mode".to_string());
            return Ok(0);
        }

        self.push_decision("Starting 1-day simulation".to_string());

        let mut trades = 0usize;
        let steps = 78; // ~6.5 hours of 5-min candles

        for step in 0..steps {
            if step % 10 == 0 {
                self.push_decision(format!("Simulated hour block {}", (step / 10) + 1));
            }

            let price = dec!(50) + Decimal::from(rand::random::<u32>() % 60);
            let market = Market {
                id: format!("sim_market_{}", step),
                name: format!("Simulated Market #{}", step),
                description: "Simulated day market".to_string(),
                current_price: price,
                volume: dec!(10000),
                liquidity: dec!(50000),
                history: vec![dec!(45), dec!(48), dec!(52), price],
                category: "Stocks".to_string(),
                expiry_date: Some(Utc::now() + chrono::Duration::days(30)),
            };

            let fair_value = self.estimate_fair_value(&market).await?;
            let price_diff = if market.current_price > fair_value {
                market.current_price - fair_value
            } else {
                fair_value - market.current_price
            };
            let mispricing = (price_diff / fair_value) * dec!(100);

            if mispricing > settings.min_mispricing_percent {
                let position = if market.current_price < fair_value {
                    Position::Long
                } else {
                    Position::Short
                };

                let position_size = (self.balance * settings.max_position_percent).max(dec!(5));
                if self.execute_trade(market, position, position_size, price).await.is_ok() {
                    trades += 1;
                }
            }
        }

        self.push_decision(format!("Day simulation complete: {} trades", trades));
        Ok(trades)
    }

    fn push_decision(&mut self, message: String) {
        if self.decision_log.len() >= 200 {
            self.decision_log.pop_front();
        }
        self.decision_log.push_back(message);
    }
    
    pub fn add_news_source(&mut self, source: String) {
        if !self.news_sources.contains(&source) {
            self.news_sources.push(source);
            info!("➕ Added news source");
        }
    }
    
    pub fn get_stats(&self) -> TraderStats {
        TraderStats {
            balance: self.balance,
            total_profit: self.total_profit,
            total_loss: self.total_loss,
            trades_executed: self.trades_executed,
            is_alive: self.is_alive,
            is_test_mode: self.is_test_mode,
            simulate_markets: self.simulate_markets,
            is_paused: self.is_paused,
            preferred_categories: self.preferred_categories.clone(),
            cycle_count: self.cycle_count,
            portfolio_size: self.portfolio.len(),
            news_sources_count: self.news_sources.len(),
        }
    }
}

pub struct TraderStats {
    pub balance: Decimal,
    pub total_profit: Decimal,
    pub total_loss: Decimal,
    pub trades_executed: usize,
    pub is_alive: bool,
    pub is_test_mode: bool,
    pub simulate_markets: bool,
    pub is_paused: bool,
    pub preferred_categories: Vec<String>,
    pub cycle_count: u32,
    pub portfolio_size: usize,
    pub news_sources_count: usize,
}

impl Trader {
    pub async fn execute_trades(&mut self, _settings: &Settings) -> Result<Decimal> {
        use rust_decimal_macros::dec;
        use chrono::Utc;

        if self.trades_executed % 3 == 0 {
            let random_num = rand::random::<u32>() % 30;
            let price = dec!(50) + Decimal::from(random_num);
            let market = Market {
                id: "auto_trade".to_string(),
                name: "Auto Trading Opportunity".to_string(),
                description: "Market found during scanning".to_string(),
                current_price: price,
                volume: dec!(10000),
                liquidity: dec!(50000),
                history: vec![dec!(45), dec!(48), dec!(52), dec!(50)],
                category: "Auto".to_string(),
                expiry_date: Some(Utc::now() + chrono::Duration::days(30)),
            };

            return self.execute_trade(market, Position::Long, dec!(10), price).await;
        }

        Ok(dec!(0))
    }
}
