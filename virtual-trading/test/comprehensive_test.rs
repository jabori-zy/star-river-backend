use virtual_trading::VirtualTradingSystem;
use event_center::EventCenter;
use tokio::sync::watch;
use types::market::Exchange;
use types::order::{FuturesOrderSide, OrderType};
use types::order::virtual_order::VirtualOrder;
use types::position::virtual_position::VirtualPosition;
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
    
    system.set_initial_balance(10000.0);
    system.set_leverage(10);
    system.set_fee_rate(0.0005);
    
    system
}

#[tokio::test]
async fn test_complete_trading_scenario() {
    let mut system = create_test_system().await;
    
    println!("=== 完整交易场景测试 ===");
    println!("初始资金: {} USDT", system.get_initial_balance());
    println!("杠杆倍数: {}倍", system.get_leverage());
    println!("手续费率: {}%", system.fee_rate * 100.0);
    println!();
    
    // 场景1: 创建多个订单并检查保证金需求
    let orders_config = vec![
        ("BTCUSDT", 0.1, 50000.0, FuturesOrderSide::OpenLong),
        ("ETHUSDT", 1.0, 3000.0, FuturesOrderSide::OpenLong),
        ("ADAUSDT", 1000.0, 1.5, FuturesOrderSide::OpenShort),
    ];
    
    let mut total_margin_needed = 0.0;
    
    println!("📋 订单保证金需求分析:");
    for (symbol, quantity, price, side) in &orders_config {
        let order = VirtualOrder {
            order_id: 0,
            strategy_id: 1,
            node_id: "test".to_string(),
            order_config_id: 1,
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            order_side: side.clone(),
            order_type: OrderType::Market,
            quantity: *quantity,
            open_price: *price,
            tp: None,
            sl: None,
            order_status: types::order::OrderStatus::Created,
            position_id: None,
            create_time: Utc::now(),
            update_time: Utc::now(),
        };
        
        let margin = system.calculate_margin(*price, *quantity);
        let margin_ratio = system.calculate_margin_ratio(*price, *quantity);
        total_margin_needed += margin;
        
        println!("  {} {:?}: {} × {} = {} USDT保证金 ({}%)", 
                 symbol, side, quantity, price, margin, margin_ratio * 100.0);
    }
    
    println!("  总保证金需求: {} USDT", total_margin_needed);
    println!("  剩余可用资金: {} USDT", system.get_current_balance() - total_margin_needed);
    println!();
    
    // 场景2: 模拟价格波动对仓位的影响
    println!("📈 价格波动影响分析:");
    
    // 创建仓位
    let mut positions = Vec::new();
    for (i, (symbol, quantity, price, side)) in orders_config.iter().enumerate() {
        let order = VirtualOrder {
            order_id: i as i32,
            strategy_id: 1,
            node_id: "test".to_string(),
            order_config_id: 1,
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            order_side: side.clone(),
            order_type: OrderType::Market,
            quantity: *quantity,
            open_price: *price,
            tp: None,
            sl: None,
            order_status: types::order::OrderStatus::Created,
            position_id: None,
            create_time: Utc::now(),
            update_time: Utc::now(),
        };
        
        let margin = system.calculate_margin(*price, *quantity);
        let force_price = system.calculate_force_price(*price, *quantity);
        
        let mut position = VirtualPosition::new(&order, *price, force_price, margin, margin / system.current_balance, 1640995200000);
        positions.push(position);
    }
    
    // 模拟价格变动
    let price_scenarios = vec![
        ("乐观场景", vec![55000.0, 3300.0, 1.4]), // +10%, +10%, -6.7%
        ("中性场景", vec![50000.0, 3000.0, 1.5]), // 0%, 0%, 0%
        ("悲观场景", vec![45000.0, 2700.0, 1.6]), // -10%, -10%, +6.7%
        ("极端场景", vec![40000.0, 2400.0, 1.8]), // -20%, -20%, +20%
    ];
    
    for (scenario_name, prices) in &price_scenarios {
        println!("  {} 场景:", scenario_name);
        let mut total_pnl = 0.0;
        
        for (i, &new_price) in prices.iter().enumerate() {
            let original_price = match i {
                0 => 50000.0, // BTC
                1 => 3000.0,  // ETH
                2 => 1.5,     // ADA
                _ => 0.0,
            };

            let margin = system.calculate_margin(new_price, positions[i].quantity);
            let margin_ratio = system.calculate_margin_ratio(new_price, positions[i].quantity);
            let force_price = system.calculate_force_price(new_price, positions[i].quantity);
            positions[i].update_position(new_price, system.get_timestamp(), margin, margin_ratio, force_price);
            let pnl = positions[i].unrealized_profit;
            
            // 对于空头，需要调整计算逻辑
            let corrected_pnl = if matches!(orders_config[i].3, FuturesOrderSide::OpenShort) {
                orders_config[i].1 * (original_price - new_price)
            } else {
                pnl
            };
            
            total_pnl += corrected_pnl;
            
            let pnl_percentage = (corrected_pnl / positions[i].margin) * 100.0;
            println!("    {}: {} → {} (盈亏: {} USDT, {}%)", 
                     orders_config[i].0, original_price, new_price, corrected_pnl, pnl_percentage);
        }
        
        let total_equity = system.current_balance + total_pnl;
        let return_rate = (total_pnl / total_margin_needed) * 100.0;
        println!("    📊 总盈亏: {} USDT, 总权益: {} USDT, 收益率: {:.1}%", total_pnl, total_equity, return_rate);
        println!();
    }
}

#[tokio::test]
async fn test_risk_management_scenarios() {
    let mut system = create_test_system().await;
    
    println!("=== 风险管理场景测试 ===");
    
    // 测试高风险交易
    system.set_leverage(100); // 设置100倍杠杆
    
    let high_risk_order = VirtualOrder {
        order_id: 1,
        strategy_id: 1,
        node_id: "test".to_string(),
        order_config_id: 1,
        exchange: Exchange::Binance,
        symbol: "BTCUSDT".to_string(),
        order_side: FuturesOrderSide::OpenLong,
        order_type: OrderType::Market,
        quantity: 1.0, // 1个BTC
        open_price: 50000.0,
        tp: None,
        sl: None,
        order_status: types::order::OrderStatus::Created,
        position_id: None,
        create_time: Utc::now(),
        update_time: Utc::now(),
    };
    
    let margin = system.calculate_margin(50000.0, 1.0);
    let force_price = system.calculate_force_price(50000.0, 1.0);
    let liquidation_distance = ((50000.0 - force_price) / 50000.0) * 100.0;
    
    println!("🔥 高风险交易分析 (100倍杠杆):");
    println!("  仓位规模: 1 BTC @ 50000 USDT");
    println!("  保证金: {} USDT", margin);
    println!("  强平价格: {} USDT", force_price);
    println!("  强平距离: {:.2}%", liquidation_distance);
    
    // 测试小幅价格波动的影响
    println!("\n  价格波动影响:");
    let small_changes = [-0.005, -0.002, -0.001, 0.001, 0.002, 0.005]; // ±0.1% to ±0.5%
    
    for change in small_changes {
        let new_price = 50000.0 * (1.0 + change);
        let pnl = 1.0 * (new_price - 50000.0);
        let roi = (pnl / margin) * 100.0;
        println!("    价格变动 {:+.1}%: 盈亏 {} USDT, ROI {:+.1}%", 
                 change * 100.0, pnl, roi);
    }
}

#[tokio::test]
async fn test_account_equity_calculation() {
    let mut system = create_test_system().await;
    
    println!("=== 账户权益计算测试 ===");
    
    // 创建多个仓位
    let positions_data = vec![
        ("BTCUSDT", 0.1, 50000.0, 52000.0), // 盈利
        ("ETHUSDT", 1.0, 3000.0, 2900.0),   // 亏损
        ("ADAUSDT", 1000.0, 1.5, 1.6),      // 盈利
    ];
    
    let mut total_margin = 0.0;
    let mut total_unrealized_pnl = 0.0;
    
    println!("📊 账户权益详细计算:");
    println!("初始余额: {} USDT", system.get_initial_balance());
    println!();
    
    for (symbol, quantity, entry_price, current_price) in &positions_data {
        let order = VirtualOrder {
            order_id: 0,
            strategy_id: 1,
            node_id: "test".to_string(),
            order_config_id: 1,
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            order_side: FuturesOrderSide::OpenLong,
            order_type: OrderType::Market,
            quantity: *quantity,
            open_price: *entry_price,
            tp: None,
            sl: None,
            order_status: types::order::OrderStatus::Created,
            position_id: None,
            create_time: Utc::now(),
            update_time: Utc::now(),
        };
        
        let margin = system.calculate_margin(*entry_price, *quantity);
        let margin_ratio = system.calculate_margin_ratio(*entry_price, *quantity);
        let force_price = system.calculate_force_price(*entry_price, *quantity);
        let mut position = VirtualPosition::new(&order, *entry_price, force_price, margin, margin_ratio, system.get_timestamp());
        position.update_position(*current_price, system.get_timestamp(), margin, margin_ratio, force_price);
        
        total_margin += margin;
        total_unrealized_pnl += position.unrealized_profit;
        
        let pnl_percentage = (position.unrealized_profit / margin) * 100.0;
        println!("{}:", symbol);
        println!("  数量: {}", quantity);
        println!("  开仓价: {} → 当前价: {}", entry_price, current_price);
        println!("  保证金: {} USDT", margin);
        println!("  未实现盈亏: {} USDT ({}%)", position.unrealized_profit, pnl_percentage);
        println!();
        
        system.current_positions.push(position);
    }
    
    system.update_unrealized_pnl();
    
    let current_balance = system.get_initial_balance() - total_margin; // 扣除保证金后的可用余额
    let total_equity = current_balance + total_margin + system.unrealized_pnl; // 总权益
    let total_return = (system.unrealized_pnl / system.get_initial_balance()) * 100.0;
    
    println!("📋 账户权益总结:");
    println!("可用余额: {} USDT", current_balance);
    println!("占用保证金: {} USDT", total_margin);
    println!("未实现盈亏: {} USDT", system.unrealized_pnl);
    println!("总权益: {} USDT", total_equity);
    println!("总收益率: {:.2}%", total_return);
    
    // 风险指标
    let margin_ratio = (total_margin / total_equity) * 100.0;
    let available_margin_ratio = (current_balance / total_equity) * 100.0;
    
    println!("\n📈 风险指标:");
    println!("保证金率: {:.1}%", margin_ratio);
    println!("可用保证金率: {:.1}%", available_margin_ratio);
    
    if margin_ratio > 80.0 {
        println!("⚠️ 高风险: 保证金率过高");
    } else if margin_ratio > 60.0 {
        println!("⚠️ 中风险: 保证金率较高");
    } else {
        println!("✅ 低风险: 保证金率正常");
    }
}