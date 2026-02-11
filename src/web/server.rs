use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::agent::Trader;
use crate::config::Settings;
use crate::llm;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use log::info;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ControlPanel {
    balance: f64,
    total_profit: f64,
    total_loss: f64,
    trades_executed: usize,
    is_alive: bool,
    is_test_mode: bool,
    simulate_markets: bool,
    is_paused: bool,
    preferred_categories: Vec<String>,
    cycle_count: u32,
    portfolio_size: usize,
    news_sources_count: usize,
}

#[derive(Serialize)]
struct HoldingSummary {
    market_name: String,
    net_amount: f64,
    trades: usize,
    net_profit_loss: f64,
}

#[derive(Serialize)]
struct ProfitLossItem {
    market_name: String,
    profit_loss: f64,
    amount: f64,
    timestamp: String,
}

pub async fn start_web_ui(trader: Arc<Mutex<Trader>>, settings: Settings) -> Result<()> {
    let trader_filter = warp::any().map(move || Arc::clone(&trader));
    let settings_filter = warp::any().map(move || settings.clone());
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);
    
    let dashboard = warp::path("api")
        .and(warp::path("dashboard"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_dashboard);
    
    let toggle_test = warp::path("api")
        .and(warp::path("toggle-test"))
        .and(warp::post())
        .and(trader_filter.clone())
        .and_then(handle_toggle_test);

    let toggle_simulation = warp::path("api")
        .and(warp::path("toggle-sim"))
        .and(warp::post())
        .and(trader_filter.clone())
        .and_then(handle_toggle_simulation);

    let toggle_running = warp::path("api")
        .and(warp::path("toggle-run"))
        .and(warp::post())
        .and(trader_filter.clone())
        .and_then(handle_toggle_running);
    
    let add_news_source = warp::path("api")
        .and(warp::path("add-news"))
        .and(warp::post())
        .and(warp::body::json())
        .and(trader_filter.clone())
        .and_then(handle_add_news);
    
    let force_trade = warp::path("api")
        .and(warp::path("force-trade"))
        .and(warp::post())
        .and(trader_filter.clone())
        .and_then(handle_force_trade);
    
    let kill_agent = warp::path("api")
        .and(warp::path("kill"))
        .and(warp::post())
        .and(trader_filter.clone())
        .and_then(handle_kill_agent);
    
    let reset_agent = warp::path("api")
        .and(warp::path("reset"))
        .and(warp::post())
        .and(settings_filter.clone())
        .and(trader_filter.clone())
        .and_then(handle_reset_agent);
    
    let trade_history = warp::path("api")
        .and(warp::path("trades"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_trade_history);

    let portfolio = warp::path("api")
        .and(warp::path("portfolio"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_portfolio);

    let profit_loss = warp::path("api")
        .and(warp::path("profit-loss"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_profit_loss);

    let simulate_day = warp::path("api")
        .and(warp::path("simulate-day"))
        .and(warp::post())
        .and(settings_filter.clone())
        .and(trader_filter.clone())
        .and_then(handle_simulate_day);

    let news_feed = warp::path("api")
        .and(warp::path("news"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_news);

    let news_sources = warp::path("api")
        .and(warp::path("news-sources"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_news_sources);

    let decision_log = warp::path("api")
        .and(warp::path("thoughts"))
        .and(warp::get())
        .and(trader_filter.clone())
        .and_then(handle_thoughts);

    let llm_instruction = warp::path("api")
        .and(warp::path("instruct"))
        .and(warp::post())
        .and(warp::body::json())
        .and(settings_filter.clone())
        .and(trader_filter.clone())
        .and_then(handle_llm_instruction);
    
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    
    let index = warp::path::end()
        .and(warp::fs::file("static/index.html"));
    
    let routes = dashboard
        .or(toggle_test)
        .or(toggle_simulation)
        .or(toggle_running)
        .or(add_news_source)
        .or(force_trade)
        .or(kill_agent)
        .or(reset_agent)
        .or(trade_history)
        .or(portfolio)
        .or(profit_loss)
        .or(simulate_day)
        .or(news_feed)
        .or(news_sources)
        .or(decision_log)
        .or(llm_instruction)
        .or(static_files)
        .or(index)
        .with(cors)
        .with(warp::log("web"));
    
    info!("🌐 Web server starting on http://localhost:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
    
    Ok(())
}

async fn handle_dashboard(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    let stats = trader_lock.get_stats();
    
    let balance_f64: f64 = stats.balance.to_string().parse().unwrap_or(0.0);
    let total_profit_f64: f64 = stats.total_profit.to_string().parse().unwrap_or(0.0);
    let total_loss_f64: f64 = stats.total_loss.to_string().parse().unwrap_or(0.0);
    
    let panel = ControlPanel {
        balance: balance_f64,
        total_profit: total_profit_f64,
        total_loss: total_loss_f64,
        trades_executed: stats.trades_executed,
        is_alive: stats.is_alive,
        is_test_mode: stats.is_test_mode,
        simulate_markets: stats.simulate_markets,
        is_paused: stats.is_paused,
        preferred_categories: stats.preferred_categories,
        cycle_count: stats.cycle_count,
        portfolio_size: stats.portfolio_size,
        news_sources_count: stats.news_sources_count,
    };
    
    Ok(warp::reply::json(&panel))
}

async fn handle_toggle_test(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    trader_lock.toggle_test_mode();
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "test_mode": trader_lock.is_test_mode
    })))
}

async fn handle_toggle_simulation(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    trader_lock.toggle_simulation();

    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "simulate_markets": trader_lock.simulate_markets,
        "test_mode": trader_lock.is_test_mode
    })))
}

async fn handle_toggle_running(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    trader_lock.toggle_running();

    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "is_paused": trader_lock.is_paused
    })))
}

async fn handle_add_news(
    source: String,
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    trader_lock.add_news_source(source.clone());
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": format!("Added news source: {}", source),
        "total_sources": trader_lock.news_sources.len()
    })))
}

async fn handle_force_trade(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    
    use crate::agent::Market;
    use rust_decimal_macros::dec;
    use chrono::Utc;
    
    let market = Market {
        id: "forced_trade_market".to_string(),
        name: "Forced Test Market".to_string(),
        description: "Market created for forced trade testing".to_string(),
        current_price: dec!(50),
        volume: dec!(10000),
        liquidity: dec!(50000),
        history: vec![dec!(45), dec!(48), dec!(52), dec!(50)],
        category: "Test".to_string(),
        expiry_date: Some(Utc::now() + chrono::Duration::days(30)),
    };
    
    match trader_lock.execute_trade(
        market,
        crate::agent::Position::Long,
        dec!(10),
        dec!(50),
    ).await {
        Ok(profit) => {
            let profit_f64: f64 = profit.to_string().parse().unwrap_or(0.0);
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "message": format!("Forced trade executed with profit: ${}", profit),
                "profit": profit_f64
            })))
        },
        Err(e) => {
            Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": format!("Failed to execute trade: {}", e)
            })))
        }
    }
}

async fn handle_kill_agent(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    trader_lock.balance = rust_decimal_macros::dec!(0);
    trader_lock.is_alive = false;
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Agent killed (simulated). Balance set to $0."
    })))
}

async fn handle_reset_agent(
    settings: Settings,
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    *trader_lock = Trader::from_settings(&settings);
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Agent reset to initial state"
    })))
}

async fn handle_trade_history(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    let history: Vec<_> = trader_lock.trade_history.iter().cloned().collect();
    
    Ok(warp::reply::json(&history))
}

async fn handle_portfolio(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    let mut by_market: HashMap<String, HoldingSummary> = HashMap::new();

    for trade in trader_lock.trade_history.iter() {
        let entry = by_market.entry(trade.market_name.clone()).or_insert(HoldingSummary {
            market_name: trade.market_name.clone(),
            net_amount: 0.0,
            trades: 0,
            net_profit_loss: 0.0,
        });

        let amount = trade.amount.to_string().parse::<f64>().unwrap_or(0.0);
        let direction = match trade.position {
            crate::agent::Position::Short | crate::agent::Position::No => -1.0,
            _ => 1.0,
        };

        entry.net_amount += amount * direction;
        entry.trades += 1;
        let pl = trade.profit_loss.unwrap_or_default().to_string().parse::<f64>().unwrap_or(0.0);
        entry.net_profit_loss += pl;
    }

    let mut holdings: Vec<_> = by_market.into_values().collect();
    holdings.sort_by(|a, b| b.net_profit_loss.partial_cmp(&a.net_profit_loss).unwrap_or(std::cmp::Ordering::Equal));

    Ok(warp::reply::json(&holdings))
}

async fn handle_profit_loss(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    let mut items: Vec<ProfitLossItem> = trader_lock.trade_history.iter().map(|trade| {
        let profit_loss = trade.profit_loss.unwrap_or_default().to_string().parse::<f64>().unwrap_or(0.0);
        let amount = trade.amount.to_string().parse::<f64>().unwrap_or(0.0);
        ProfitLossItem {
            market_name: trade.market_name.clone(),
            profit_loss,
            amount,
            timestamp: trade.timestamp.to_rfc3339(),
        }
    }).collect();

    items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(warp::reply::json(&items))
}

async fn handle_simulate_day(
    settings: Settings,
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let mut trader_lock = trader.lock().await;
    let trades = trader_lock.simulate_day(&settings).await.unwrap_or(0);

    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "trades": trades
    })))
}

async fn handle_news(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    Ok(warp::reply::json(&trader_lock.last_news))
}

async fn handle_news_sources(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    Ok(warp::reply::json(&trader_lock.news_sources))
}

async fn handle_thoughts(
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let trader_lock = trader.lock().await;
    let thoughts: Vec<_> = trader_lock.decision_log.iter().cloned().collect();
    Ok(warp::reply::json(&thoughts))
}

#[derive(Serialize, Deserialize)]
struct LlmInstructionPayload {
    instruction: String,
}

async fn handle_llm_instruction(
    payload: LlmInstructionPayload,
    settings: Settings,
    trader: Arc<Mutex<Trader>>
) -> Result<impl Reply, Rejection> {
    let response = llm::run_instruction(&settings, &payload.instruction).await;

    let mut trader_lock = trader.lock().await;
    trader_lock.last_instruction = Some(payload.instruction.clone());
    let categories = trader_lock.apply_instruction(&payload.instruction);

    match response {
        Ok(text) => {
            trader_lock.last_llm_response = Some(text.clone());
            Ok(warp::reply::json(&serde_json::json!({
                "status": "success",
                "response": text,
                "preferred_categories": categories
            })))
        }
        Err(e) => {
            trader_lock.last_llm_response = Some(format!("Error: {}", e));
            Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": format!("LLM instruction failed: {}", e)
            })))
        }
    }
}
