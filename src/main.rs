use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

mod agent;
mod market;
mod news;
mod config;
mod web;
mod llm;

use crate::agent::Trader;
use crate::config::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("🚀 Starting AI Trading Agent - Survival Mode");
    log::info!("💵 Starting with $50 - Pay for yourself or die!");
    
    // Load configuration
    let settings = Settings::new()?;
    log::info!("✅ Configuration loaded");
    
    // Initialize agent with $50 starting capital
    let starting_balance = settings.starting_balance;
    let trader = Arc::new(Mutex::new(Trader::from_settings(&settings)));
    log::info!("🤖 Agent initialized with ${}", starting_balance);
    
    // Start web UI for monitoring and control
    let trader_clone = Arc::clone(&trader);
    let web_settings = settings.clone();
    tokio::spawn(async move {
        if let Err(e) = web::server::start_web_ui(trader_clone, web_settings).await {
            log::error!("Failed to start web UI: {}", e);
        }
    });
    
    log::info!("🌐 Web dashboard available at http://localhost:3030");
    log::info!("⏰ Starting 10-minute trading cycles...");
    
    // Run the main trading loop
    run_trading_loop(trader, settings).await?;
    
    Ok(())
}

async fn run_trading_loop(
    trader: Arc<Mutex<Trader>>,
    settings: Settings
) -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600)); // Every 10 minutes
    
    let mut cycle_count = 0;
    
    loop {
        interval.tick().await;
        cycle_count += 1;
        
        log::info!("🔄 Starting trading cycle #{}", cycle_count);
        
        let mut trader_lock = trader.lock().await;
        
        // Check if agent is alive
        if !trader_lock.is_alive() {
            log::error!("💀 Agent has died - balance reached $0");
            log::error!("📊 Final Stats:");
            log::error!("  Total Profit: ${}", trader_lock.total_profit);
            log::error!("  Total Loss: ${}", trader_lock.total_loss);
            log::error!("  Trades Executed: {}", trader_lock.trades_executed);
            break;
        }
        
        // Pay API bill from profits (simulated)
        trader_lock.pay_api_bill(&settings);
        
        // Scan markets (500-1000 markets as described)
        match trader_lock.scan_markets(&settings).await {
            Ok(_) => log::info!("✅ Market scan completed"),
            Err(e) => log::error!("❌ Market scan failed: {}", e),
        }
        
        // Update news analysis
        match trader_lock.update_news_analysis(&settings).await {
            Ok(_) => log::info!("✅ News analysis updated"),
            Err(e) => log::error!("❌ News analysis failed: {}", e),
        }
        
        // Execute trades based on findings
        match trader_lock.execute_trades(&settings).await {
            Ok(profit) => {
                if profit != rust_decimal_macros::dec!(0) {
                    log::info!("💰 Trade executed with profit: ${}", profit);
                }
            },
            Err(e) => log::error!("❌ Trade execution failed: {}", e),
        }
        
        // Log current status
        trader_lock.log_status();
        
        log::info!("✅ Completed trading cycle #{}. Next in 10 minutes...", cycle_count);
        log::info!("---");
        
        // Drop lock for next iteration
        drop(trader_lock);
    }
    
    Ok(())
}
