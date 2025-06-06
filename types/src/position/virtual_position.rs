use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::custom_type::*;
use crate::market::Exchange;
use crate::position::{PositionSide, PositionState};
use crate::order::virtual_order::VirtualOrder;
use crate::order::OrderSide;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPosition {
    pub position_id: i32,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_state: PositionState, // 持仓状态
    pub quantity: f64,
    pub open_price: f64,
    pub current_price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub unrealized_profit: Option<f64>, // 未实现盈亏
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

impl VirtualPosition {
    pub fn new(virtual_order: &VirtualOrder, current_price: f64) -> Self {
        let position_side = match virtual_order.order_side {
            OrderSide::Long => PositionSide::Long,
            OrderSide::Short => PositionSide::Short,
        };

        Self {
            position_id: virtual_order.order_id,
            strategy_id: virtual_order.strategy_id,
            node_id: virtual_order.node_id.clone(),
            exchange: virtual_order.exchange.clone(),
            symbol: virtual_order.symbol.clone(),
            position_side,
            position_state: PositionState::Open,
            quantity: virtual_order.quantity,
            open_price: current_price,
            current_price: current_price,
            tp: virtual_order.tp,
            sl: virtual_order.sl,
            unrealized_profit: None,
            create_time: virtual_order.created_time,
            update_time: virtual_order.updated_time,
        }
    }

    
}