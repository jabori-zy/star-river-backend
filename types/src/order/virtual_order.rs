use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::order::{OrderSide,OrderType,OrderStatus};
use crate::market::Exchange;
use crate::custom_type::*;
use crate::position::virtual_position::VirtualPosition;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualOrder {
    pub order_id: OrderId, // 订单ID
    pub strategy_id: StrategyId, // 策略ID
    pub node_id: NodeId, // 节点ID
    pub exchange: Exchange, // 交易所
    pub symbol: String, // 交易对
    pub order_side: OrderSide, // 订单方向
    pub order_type: OrderType, // 订单类型
    pub order_status: OrderStatus, // 订单状态
    pub quantity: f64, // 数量
    pub open_price: f64, // 开仓价格
    pub tp: Option<f64>, // 止盈价格
    pub sl: Option<f64>, // 止损价格
    pub created_time: DateTime<Utc>, // 创建时间
    pub updated_time: DateTime<Utc>, // 更新时间
}

impl VirtualOrder {
    pub fn new(
        order_id: OrderId,
        strategy_id: StrategyId,
        node_id: NodeId,
        exchange: Exchange,
        symbol: String,
        order_side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        open_price: f64,
        tp: Option<f64>,
        sl: Option<f64>,
    ) -> Self {
        Self {
            order_id,
            strategy_id,
            node_id,
            exchange,
            symbol,
            order_side,
            order_type,
            quantity,
            open_price,
            tp,
            sl,
            order_status: OrderStatus::Created,
            created_time: Utc::now(),
            updated_time: Utc::now(),
        }
    }

    // 成交, 返回成交的仓位

}

