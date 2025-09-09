use crate::order::virtual_order::VirtualOrder;
use crate::strategy::node_event::BacktestNodeEvent;
use crate::transaction::virtual_transaction::VirtualTransaction;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_type")]
pub enum FuturesOrderNodeEvent {
    #[strum(serialize = "futures-order-created")]
    #[serde(rename = "futures-order-created")]
    FuturesOrderCreated(FuturesOrderCreatedEvent), // 订单已创建

    #[strum(serialize = "futures-order-canceled")]
    #[serde(rename = "futures-order-canceled")]
    FuturesOrderCanceled(FuturesOrderCanceledEvent), // 订单已取消

    #[strum(serialize = "futures-order-filled")]
    #[serde(rename = "futures-order-filled")]
    FuturesOrderFilled(FuturesOrderFilledEvent), // 订单已成交

    #[strum(serialize = "take-profit-order-created")]
    #[serde(rename = "take-profit-order-created")]
    TakeProfitOrderCreated(TakeProfitOrderCreatedEvent), // 止盈订单已创建

    #[strum(serialize = "take-profit-order-filled")]
    #[serde(rename = "take-profit-order-filled")]
    TakeProfitOrderFilled(TakeProfitOrderFilledEvent), // 止盈订单已成交

    #[strum(serialize = "take-profit-order-canceled")]
    #[serde(rename = "take-profit-order-canceled")]
    TakeProfitOrderCanceled(TakeProfitOrderCanceledEvent), // 止盈订单已取消

    #[strum(serialize = "stop-loss-order-created")]
    #[serde(rename = "stop-loss-order-created")]
    StopLossOrderCreated(StopLossOrderCreatedEvent), // 止损订单已创建

    #[strum(serialize = "stop-loss-order-filled")]
    #[serde(rename = "stop-loss-order-filled")]
    StopLossOrderFilled(StopLossOrderFilledEvent), // 止损订单已成交

    #[strum(serialize = "stop-loss-order-canceled")]
    #[serde(rename = "stop-loss-order-canceled")]
    StopLossOrderCanceled(StopLossOrderCanceledEvent), // 止损订单已取消

    #[strum(serialize = "transaction-created")]
    #[serde(rename = "transaction-created")]
    TransactionCreated(TransactionCreatedEvent), // 交易明细已创建
}

impl From<FuturesOrderNodeEvent> for BacktestNodeEvent {
    fn from(event: FuturesOrderNodeEvent) -> Self {
        BacktestNodeEvent::FuturesOrderNode(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderCreatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderCanceledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderFilledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitOrderCreatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrderCreatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitOrderFilledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrderFilledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitOrderCanceledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrderCanceledEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: VirtualOrder,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCreatedEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "transaction")]
    pub transaction: VirtualTransaction,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}
