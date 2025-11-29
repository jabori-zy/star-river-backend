use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::{custom_type::*, exchange::Exchange, transaction::FuturesTransSide};
use utoipa::{IntoParams, ToSchema};

use super::id_generator::TRANSACTION_ID_COUNTER;
#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VirtualTransaction {
    pub transaction_id: TransactionId, // 交易明细id

    pub order_id: OrderId, // 订单id

    pub position_id: PositionId, // 持仓id

    pub strategy_id: StrategyId, // 策略id

    pub node_id: NodeId, // 节点id

    pub node_name: NodeName, // 节点名称

    pub order_config_id: i32, // 订单配置id

    pub exchange: Exchange, // 交易所

    pub symbol: String, // 交易品种

    pub transaction_side: FuturesTransSide, // 交易方向

    pub quantity: f64, // 交易数量

    pub price: f64, // 交易价格

    pub profit: Option<f64>, // 收益

    pub create_time: DateTime<Utc>, // 创建时间
}

impl VirtualTransaction {
    pub fn new(
        order_id: OrderId,
        position_id: PositionId,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        order_config_id: i32,
        exchange: Exchange,
        symbol: String,
        transaction_side: FuturesTransSide,
        quantity: f64,
        price: f64,
        profit: Option<f64>,
        datetime: DateTime<Utc>,
    ) -> Self {
        let transaction_id = TRANSACTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        Self {
            transaction_id,
            order_id,
            position_id,
            strategy_id,
            node_id,
            node_name,
            order_config_id,
            exchange,
            symbol,
            transaction_side,
            quantity,
            price,
            profit,
            create_time: datetime,
        }
    }
}
