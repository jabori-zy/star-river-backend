use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::custom_type::*;
use crate::market::Exchange;
use crate::transaction::TransactionType;
use crate::transaction::TransactionSide;
use crate::order::virtual_order::VirtualOrder;
use crate::position::virtual_position::VirtualPosition;
use crate::order::OrderSide;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTransaction {
    pub transaction_id: TransactionId, // 交易明细id
    pub order_id: OrderId, // 订单id
    pub position_id: PositionId, // 持仓id
    pub strategy_id: StrategyId, // 策略id
    pub node_id: NodeId, // 节点id
    pub exchange: Exchange, // 交易所
    pub symbol: String, // 交易品种
    pub transaction_type: TransactionType, // 交易类型
    pub transaction_side: TransactionSide, // 交易方向
    pub quantity: f64, // 交易数量
    pub price: f64, // 交易价格
    pub tp: Option<f64>, // 止盈价格
    pub sl: Option<f64>, // 止损价格
    pub create_time: DateTime<Utc>, // 创建时间
}

impl VirtualTransaction {
    pub fn new(
        transaction_id: TransactionId,
        virtual_order: &VirtualOrder,
        virtual_position: &VirtualPosition,
    ) -> Self {
        Self {
            transaction_id,
            order_id: virtual_order.order_id,
            position_id: virtual_position.position_id,
            strategy_id: virtual_order.strategy_id,
            node_id: virtual_order.node_id.clone(),
            exchange: virtual_order.exchange.clone(),
            symbol: virtual_order.symbol.clone(),
            transaction_type: TransactionType::Open,
            transaction_side: match virtual_order.order_side {
                OrderSide::Long => TransactionSide::Long,
                OrderSide::Short => TransactionSide::Short,
            },
            quantity: virtual_order.quantity,
            price: virtual_position.open_price,
            tp: virtual_position.tp,
            sl: virtual_position.sl,
            create_time: Utc::now(),
        }

    }
}