use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::order::OrderSide;
use crate::order::OrderType;
use crate::order::OrderStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualOrder {
    pub order_id: i32, // 订单ID
    pub strategy_id: i32, // 策略ID
    pub node_id: String, // 节点ID
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