use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::custom_type::*;
use crate::market::Exchange;
use crate::position::{PositionSide, PositionState};
use crate::order::virtual_order::VirtualOrder;
use crate::order::FuturesOrderSide;
use utoipa::ToSchema;


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VirtualPosition {
    #[serde(rename = "positionId")]
    pub position_id: PositionId,

    #[serde(rename = "orderId")]
    pub order_id: OrderId,

    #[serde(rename = "strategyId")]
    pub strategy_id: StrategyId,

    #[serde(rename = "nodeId")]
    pub node_id: NodeId,

    #[serde(rename = "exchange")]
    pub exchange: Exchange,

    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "positionSide")]
    pub position_side: PositionSide,

    #[serde(rename = "positionState")]
    pub position_state: PositionState, // 持仓状态

    #[serde(rename = "quantity")]
    pub quantity: f64,

    #[serde(rename = "openPrice")]
    pub open_price: f64,

    #[serde(rename = "currentPrice")]
    pub current_price: f64,

    #[serde(rename = "tp")]
    pub tp: Option<f64>,

    #[serde(rename = "sl")]
    pub sl: Option<f64>,

    #[serde(rename = "unrealizedProfit")]
    pub unrealized_profit: f64, // 未实现盈亏

    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,

    #[serde(rename = "updateTime")]
    pub update_time: DateTime<Utc>,
}

impl VirtualPosition {
    pub fn new(virtual_order: &VirtualOrder, current_price: f64, timestamp: i64) -> Self {

        let position_side = match virtual_order.order_side {
            FuturesOrderSide::OpenLong => PositionSide::Long,
            FuturesOrderSide::OpenShort => PositionSide::Short,
            FuturesOrderSide::CloseLong => PositionSide::Long,
            FuturesOrderSide::CloseShort => PositionSide::Short,
        };

        Self {
            position_id: virtual_order.order_id,
            order_id: virtual_order.order_id,
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
            unrealized_profit: 0.0,
            create_time: DateTime::from_timestamp_millis(timestamp).unwrap(),
            update_time: DateTime::from_timestamp_millis(timestamp).unwrap(),
        }
    }

    pub fn update_position(&mut self, current_price: f64) {
        self.current_price = current_price;
        self.update_time = Utc::now();
        self.unrealized_profit = self.quantity * (current_price - self.open_price);
    }
}