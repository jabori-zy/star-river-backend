use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    position::{PositionSide, PositionState},
};
use utoipa::ToSchema;

use super::order::VirtualOrder;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VirtualPosition {
    #[serde(rename = "positionId")]
    pub position_id: PositionId,

    #[serde(rename = "orderId")]
    pub order_id: OrderId,

    #[serde(rename = "orderConfigId")]
    pub order_config_id: i32,

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
    pub open_price: Price,

    #[serde(rename = "currentPrice")]
    pub current_price: Price,

    #[serde(rename = "tp")]
    pub tp: Option<Tp>,

    #[serde(rename = "sl")]
    pub sl: Option<Sl>,

    #[serde(rename = "unrealizedProfit")]
    pub unrealized_profit: Pnl, // 未实现盈亏

    #[serde(rename = "forcePrice")]
    pub force_price: f64, // 强平价格

    #[serde(rename = "margin")]
    pub margin: Margin, // 仓位占用的保证金

    #[serde(rename = "marginRatio")]
    pub margin_ratio: MarginRatio, // 保证金率

    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,

    #[serde(rename = "updateTime")]
    pub update_time: DateTime<Utc>,
}

impl VirtualPosition {
    pub fn new(
        position_id: PositionId,
        position_side: PositionSide,
        virtual_order: &VirtualOrder,
        current_price: Price,
        force_price: Price,
        margin: Margin,
        margin_ratio: MarginRatio,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            position_id,
            order_id: virtual_order.order_id,
            order_config_id: virtual_order.order_config_id,
            strategy_id: virtual_order.strategy_id,
            node_id: virtual_order.node_id.clone(),
            exchange: virtual_order.exchange.clone(),
            symbol: virtual_order.symbol.clone(),
            position_side,
            position_state: PositionState::Open,
            quantity: virtual_order.quantity,
            open_price: current_price,
            current_price,
            tp: virtual_order.tp,
            sl: virtual_order.sl,
            unrealized_profit: 0.0,
            force_price,
            margin,
            margin_ratio,
            create_time: datetime,
            update_time: datetime,
        }
    }

    pub fn update(&mut self, current_price: Price, datetime: DateTime<Utc>, margin: Margin, margin_ratio: MarginRatio, force_price: Price) {
        self.current_price = current_price;
        self.update_time = datetime;
        self.unrealized_profit = match self.position_side {
            PositionSide::Long => self.quantity * (current_price - self.open_price),
            PositionSide::Short => self.quantity * (self.open_price - current_price),
        };
        self.margin = margin;
        self.margin_ratio = margin_ratio;
        self.force_price = force_price;
    }
}
