#[cfg(test)]
mod tests {
    use chrono::Utc;
    use star_river_core::{
        exchange::Exchange,
        order::{FuturesOrderSide, OrderType},
        position::PositionSide,
    };

    use crate::types::{VirtualOrder, VirtualPosition};

    #[test]
    fn test_update_with_new_order_close_all() {
        // Create initial long position
        // symbol=btcusdt, exchange=binance, open_price=100000, quantity=0.5
        let datetime = Utc::now();
        let mut position = VirtualPosition::new(
            PositionSide::Long,
            1, // strategy_id
            Exchange::Binance,
            "btcusdt".to_string(),
            0.5,      // quantity
            100000.0, // current_price (open_price)
            90000.0,  // force_price
            5000.0,   // margin
            0.1,      // margin_ratio
            10,       // leverage (u32)
            datetime,
        );

        // Create a short order to fully close the long position
        // Short order with same quantity (0.5) should fully close the position
        let close_order = VirtualOrder::create_order(
            1,                       // strategy_id
            "test_node".to_string(), // node_id
            "Test Node".to_string(), // node_name
            1,                       // order_config_id
            Exchange::Binance,
            "btcusdt".to_string(),
            FuturesOrderSide::Short, // opposite direction to close
            OrderType::Market,
            0.5,      // quantity - same as position
            110000.0, // open_price (close price)
            None,     // tp
            None,     // sl
            None,     // tp_type
            None,     // sl_type
            None,     // point
            datetime,
        );

        // Execute close order
        let result = position.update_with_new_order(
            &close_order,
            110000.0, // current_price (actual close price)
            10000.0,  // available_balance
            datetime,
        );

        // Verify result
        assert!(result.is_ok());
        let (_position_state, transaction) = result.unwrap();

        // Position should be fully closed
        // assert!(matches!(position_state, PositionState::Closed));
        assert_eq!(position.quantity, 0.0);
        assert_eq!(position.margin, 0.0);
        assert_eq!(position.margin_ratio, 0.0);
        assert_eq!(position.force_price, 0.0);

        // Verify realized profit
        // Long position: profit = quantity * (close_price - open_price)
        // profit = 0.5 * (110000 - 100000) = 5000
        let expected_profit = 0.5 * (110000.0 - 100000.0);
        assert_eq!(position.unrealized_profit, expected_profit);
        assert_eq!(transaction.profit, Some(expected_profit));

        // Verify transaction details
        assert_eq!(transaction.quantity, 0.5);
        assert_eq!(transaction.price, 110000.0);
        assert_eq!(transaction.symbol, "btcusdt");
    }
}
