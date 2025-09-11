use crate::custom_type::*;
use crate::market::Exchange;
use crate::order::virtual_order::VirtualOrder;
use crate::order::FuturesOrderSide;
use crate::position::virtual_position::VirtualPosition;
use crate::transaction::TransactionSide;
use crate::transaction::TransactionType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams, ToSchema)]
pub struct VirtualTransaction {
    #[serde(rename = "transactionId")]
    pub transaction_id: TransactionId, // 交易明细id

    #[serde(rename = "orderId")]
    pub order_id: OrderId, // 订单id

    #[serde(rename = "positionId")]
    pub position_id: PositionId, // 持仓id

    #[serde(rename = "strategyId")]
    pub strategy_id: StrategyId, // 策略id

    #[serde(rename = "nodeId")]
    pub node_id: NodeId, // 节点id

    #[serde(rename = "orderConfigId")]
    pub order_config_id: i32, // 订单配置id

    #[serde(rename = "exchange")]
    pub exchange: Exchange, // 交易所

    #[serde(rename = "symbol")]
    pub symbol: String, // 交易品种

    #[serde(rename = "transactionType")]
    pub transaction_type: TransactionType, // 交易类型

    #[serde(rename = "transactionSide")]
    pub transaction_side: TransactionSide, // 交易方向

    #[serde(rename = "quantity")]
    pub quantity: f64, // 交易数量

    #[serde(rename = "price")]
    pub price: f64, // 交易价格

    #[serde(rename = "profit")]
    pub profit: Option<f64>, // 收益

    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>, // 创建时间
}

impl VirtualTransaction {
    pub fn new(
        transaction_id: TransactionId,
        virtual_order: &VirtualOrder,
        virtual_position: &VirtualPosition,
        timestamp: i64,
    ) -> Self {
        let transaction_type = match virtual_order.order_side {
            FuturesOrderSide::OpenLong => TransactionType::Open,
            FuturesOrderSide::OpenShort => TransactionType::Open,
            FuturesOrderSide::CloseLong => TransactionType::Close,
            FuturesOrderSide::CloseShort => TransactionType::Close,
        };

        let transaction_side = match virtual_order.order_side {
            FuturesOrderSide::OpenLong => TransactionSide::OpenLong,
            FuturesOrderSide::OpenShort => TransactionSide::OpenShort,
            FuturesOrderSide::CloseLong => TransactionSide::CloseLong,
            FuturesOrderSide::CloseShort => TransactionSide::CloseShort,
        };

        let profit = match transaction_type {
            TransactionType::Open => None,
            TransactionType::Close => Some(virtual_position.unrealized_profit),
        };

        Self {
            transaction_id,
            order_id: virtual_order.order_id,
            position_id: virtual_position.position_id,
            strategy_id: virtual_order.strategy_id,
            node_id: virtual_order.node_id.clone(),
            order_config_id: virtual_order.order_config_id,
            exchange: virtual_order.exchange.clone(),
            symbol: virtual_order.symbol.clone(),
            transaction_type,
            transaction_side,
            quantity: virtual_order.quantity,
            price: virtual_order.open_price,
            profit,
            create_time: DateTime::from_timestamp_millis(timestamp).unwrap(),
        }
    }
}
