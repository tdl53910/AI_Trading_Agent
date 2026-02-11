use ai_trading_agent::agent::Trader;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_trader_initialization() {
    let trader = Trader::new(dec!(50.00));
    assert_eq!(trader.balance, dec!(50.00));
    assert!(trader.is_alive);
    assert_eq!(trader.trades_executed, 0);
}

#[tokio::test]
async fn test_trader_death() {
    let mut trader = Trader::new(dec!(50.00));
    
    // Simulate massive loss
    trader.update_balance(dec!(-100.00));
    
    assert!(!trader.is_alive);
    assert_eq!(trader.balance, dec!(-50.00));
}

#[tokio::test]
async fn test_kelly_criterion() {
    use ai_trading_agent::agent::Market;
    use chrono::Utc;
    
    let trader = Trader::new(dec!(100.00));
    
    let market = Market {
        id: "test".to_string(),
        name: "Test Market".to_string(),
        description: "Test".to_string(),
        current_price: dec!(100),
        volume: dec!(1000),
        liquidity: dec!(10000),
        history: vec![dec!(90), dec!(95), dec!(105), dec!(100)],
        category: "Test".to_string(),
        expiry_date: Some(Utc::now()),
    };
    
    let kelly = trader.calculate_kelly_criterion(&market, dec!(110), dec!(10));
    
    // Kelly should be between 0 and 0.25
    assert!(kelly >= dec!(0));
    assert!(kelly <= dec!(0.25));
}

#[tokio::test]
async fn test_balance_updates() {
    let mut trader = Trader::new(dec!(100.00));
    
    // Test profit
    trader.update_balance(dec!(25.00));
    assert_eq!(trader.balance, dec!(125.00));
    assert_eq!(trader.total_profit, dec!(25.00));
    assert_eq!(trader.total_loss, dec!(0));
    
    // Test loss
    trader.update_balance(dec!(-10.00));
    assert_eq!(trader.balance, dec!(115.00));
    assert_eq!(trader.total_profit, dec!(25.00));
    assert_eq!(trader.total_loss, dec!(10.00));
}