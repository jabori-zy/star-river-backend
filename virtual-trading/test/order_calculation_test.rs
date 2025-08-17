use virtual_trading::VirtualTradingSystem;
use event_center::EventCenter;
use tokio::sync::watch;
use types::market::Exchange;
use types::order::{FuturesOrderSide, OrderType};
use types::order::virtual_order::VirtualOrder;
use chrono::Utc;

/// 创建测试用的虚拟交易系统
async fn create_test_system() -> VirtualTradingSystem {
    let event_center = EventCenter::new();
    let command_publisher = event_center.get_command_publisher();
    let (event_tx, _event_rx) = tokio::sync::broadcast::channel(100);
    let (_play_index_tx, play_index_rx) = watch::channel(0);
    
    let mut system = VirtualTradingSystem::new(
        command_publisher,
        event_tx,
        play_index_rx,
    );
    
    // 设置初始参数
    system.set_initial_balance(10000.0); // 10000 USDT
    system.set_leverage(100); // 10倍杠杆
    system.set_fee_rate(0.0005); // 0.05%手续费
    
    system
}

/// 创建测试订单
fn create_test_order(
    order_id: i32,
    symbol: &str,
    quantity: f64,
    price: f64,
    side: FuturesOrderSide,
) -> VirtualOrder {
    VirtualOrder {
        order_id,
        strategy_id: 1,
        node_id: "test".to_string(),
        order_config_id: 1,
        exchange: Exchange::Binance,
        symbol: symbol.to_string(),
        order_side: side,
        order_type: OrderType::Market,
        quantity,
        open_price: price,
        tp: None,
        sl: None,
        order_status: types::order::OrderStatus::Created,
        position_id: None,
        create_time: Utc::now(),
        update_time: Utc::now(),
    }
}

#[tokio::test]
async fn test_margin_calculation() {
    let system = create_test_system().await;
    
    // 测试BTC多头订单保证金计算
    let btc_order = create_test_order(1, "BTCUSDT", 0.1, 50000.0, FuturesOrderSide::OpenLong);
    let margin = system.calculate_margin(50000.0, 0.1);
    let expected_margin = 50000.0 * 0.1 / 100.0; // 500 USDT
    assert_eq!(margin, expected_margin);
    
    println!("✓ BTC多头保证金计算: {} USDT (预期: {} USDT)", margin, expected_margin);
    
    // 测试ETH空头订单保证金计算
    let eth_order = create_test_order(2, "ETHUSDT", 2.0, 3000.0, FuturesOrderSide::OpenShort);
    let margin = system.calculate_margin(3000.0, 2.0);
    let expected_margin = 3000.0 * 2.0 / 100.0; // 600 USDT
    assert_eq!(margin, expected_margin);
    
    println!("✓ ETH空头保证金计算: {} USDT (预期: {} USDT)", margin, expected_margin);
}

#[tokio::test]
async fn test_margin_ratio_calculation() {
    let system = create_test_system().await;
    
    let order = create_test_order(1, "BTCUSDT", 0.1, 50000.0, FuturesOrderSide::OpenLong);
    let margin_ratio = system.calculate_margin_ratio(50000.0, 0.1);
    let expected_ratio = 500.0 / 10000.0; // 5%
    assert_eq!(margin_ratio, expected_ratio);
    
    println!("✓ 保证金率计算: {}% (预期: {}%)", margin_ratio * 100.0, expected_ratio * 100.0);
}

#[tokio::test]
async fn test_force_price_calculation() {
    let system = create_test_system().await;
    
    // 测试多头强平价格
    let long_order = create_test_order(1, "BTCUSDT", 0.1, 50000.0, FuturesOrderSide::OpenLong);
    let force_price = system.calculate_force_price(50000.0, 0.1);
    let expected_force_price = 50000.0 - 500.0 / 0.1; // 45000.0
    assert_eq!(force_price, expected_force_price);
    
    println!("✓ 多头强平价格计算: {} USDT (预期: {} USDT)", force_price, expected_force_price);
    
    // 测试不同杠杆的强平价格影响
    let mut high_leverage_system = create_test_system().await;
    high_leverage_system.set_leverage(20); // 20倍杠杆
    
    let high_leverage_order = create_test_order(2, "BTCUSDT", 0.1, 50000.0, FuturesOrderSide::OpenLong);
    let high_leverage_force_price = high_leverage_system.calculate_force_price(50000.0, 0.1);
    let expected_high_leverage = 50000.0 - (50000.0 * 0.1 / 20.0) / 0.1; // 47500.0
    assert_eq!(high_leverage_force_price, expected_high_leverage);
    
    println!("✓ 高杠杆强平价格计算: {} USDT (预期: {} USDT)", high_leverage_force_price, expected_high_leverage);
}

#[tokio::test]
async fn test_insufficient_margin_check() {
    let mut system = create_test_system().await;
    
    // 设置较小的余额
    system.set_initial_balance(100.0); // 只有100 USDT
    system.current_balance = 100.0;
    
    let large_order = create_test_order(0, "BTCUSDT", 0.1, 50000.0, FuturesOrderSide::OpenLong);
    system.orders.push(large_order);
    
    // 测试保证金不足的情况
    let result = system.execute_order(0, 50000.0, 1640995200000);
    assert!(result.is_err());
    
    if let Err(error) = result {
        println!("✓ 保证金不足检查通过: {}", error);
        assert!(error.contains("保证金不足"));
    }
}

#[tokio::test]
async fn test_different_leverage_scenarios() {
    println!("=== 不同杠杆倍数保证金计算测试 ===");
    
    let leverages = [1, 5, 10, 20, 50, 100];
    let price = 50000.0;
    let quantity = 0.1;
    
    for leverage in leverages {
        let mut system = create_test_system().await;
        system.set_leverage(leverage);
        
        let order = create_test_order(1, "BTCUSDT", quantity, price, FuturesOrderSide::OpenLong);
        let margin = system.calculate_margin(price, quantity);
        let expected_margin = price * quantity / leverage as f64;
        
        assert_eq!(margin, expected_margin);
        println!("{}倍杠杆 - 保证金: {} USDT", leverage, margin);
    }
}

#[tokio::test]
async fn test_multiple_orders_margin_calculation() {
    let system = create_test_system().await;
    
    println!("=== 多订单保证金计算测试 ===");
    
    let orders = vec![
        ("BTCUSDT", 0.1, 50000.0),
        ("ETHUSDT", 2.0, 3000.0),
        ("ADAUSDT", 1000.0, 1.5),
        ("DOTUSDT", 100.0, 30.0),
    ];
    
    let mut total_expected_margin = 0.0;
    
    for (i, (symbol, quantity, price)) in orders.iter().enumerate() {
        let order = create_test_order(i as i32, symbol, *quantity, *price, FuturesOrderSide::OpenLong);
        let margin = system.calculate_margin(*price, *quantity);
        let expected_margin = price * quantity / 10.0; // 10倍杠杆
        
        assert_eq!(margin, expected_margin);
        total_expected_margin += expected_margin;
        
        println!("{}: {} × {} = {} USDT保证金", symbol, quantity, price, margin);
    }
    
    println!("总保证金需求: {} USDT", total_expected_margin);
    
    // 检查总保证金是否超过账户余额
    if total_expected_margin > system.get_current_balance() {
        println!("⚠️ 警告: 总保证金需求({} USDT)超过账户余额({} USDT)", 
                 total_expected_margin, system.get_current_balance());
    } else {
        println!("✓ 保证金需求在账户余额范围内");
    }
}