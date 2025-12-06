use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use star_river_core::{custom_type::*, exchange::Exchange, transaction::FuturesTransSide};
use utoipa::{IntoParams, ToSchema};

use super::id_generator::TRANSACTION_ID_COUNTER;
#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VirtualTransaction {
    pub transaction_id: TransactionId, // Transaction ID

    pub order_id: OrderId, // Order ID

    pub position_id: PositionId, // Position ID

    pub strategy_id: StrategyId, // Strategy ID

    pub node_id: NodeId, // Node ID

    pub node_name: NodeName, // Node name

    pub order_config_id: i32, // Order config ID

    pub exchange: Exchange, // Exchange

    pub symbol: String, // Trading symbol

    pub transaction_side: FuturesTransSide, // Transaction side

    pub quantity: f64, // Transaction quantity

    pub price: f64, // Transaction price

    pub profit: Option<f64>, // Profit

    pub create_time: DateTime<Utc>, // Create time
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
