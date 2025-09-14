use super::super::NodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum FuturesOrderNodeEvent {
    #[strum(serialize = "futures-order-created-event")]
    #[serde(rename = "futures-order-created-event")]
    FuturesOrderCreated(FuturesOrderCreatedEvent), // 订单已创建

    #[strum(serialize = "futures-order-canceled-event")]
    #[serde(rename = "futures-order-canceled-event")]
    FuturesOrderCanceled(FuturesOrderCanceledEvent), // 订单已取消

    #[strum(serialize = "futures-order-filled-event")]
    #[serde(rename = "futures-order-filled-event")]
    FuturesOrderFilled(FuturesOrderFilledEvent), // 订单已成交

    #[strum(serialize = "take-profit-order-created-event")]
    #[serde(rename = "take-profit-order-created-event")]
    TakeProfitOrderCreated(TakeProfitOrderCreatedEvent), // 止盈订单已创建

    #[strum(serialize = "take-profit-order-filled-event")]
    #[serde(rename = "take-profit-order-filled-event")]
    TakeProfitOrderFilled(TakeProfitOrderFilledEvent), // 止盈订单已成交

    #[strum(serialize = "take-profit-order-canceled-event")]
    #[serde(rename = "take-profit-order-canceled-event")]
    TakeProfitOrderCanceled(TakeProfitOrderCanceledEvent), // 止盈订单已取消

    #[strum(serialize = "stop-loss-order-created-event")]
    #[serde(rename = "stop-loss-order-created-event")]
    StopLossOrderCreated(StopLossOrderCreatedEvent), // 止损订单已创建

    #[strum(serialize = "stop-loss-order-filled-event")]
    #[serde(rename = "stop-loss-order-filled-event")]
    StopLossOrderFilled(StopLossOrderFilledEvent), // 止损订单已成交

    #[strum(serialize = "stop-loss-order-canceled-event")]
    #[serde(rename = "stop-loss-order-canceled-event")]
    StopLossOrderCanceled(StopLossOrderCanceledEvent), // 止损订单已取消

    #[strum(serialize = "transaction-created-event")]
    #[serde(rename = "transaction-created-event")]
    TransactionCreated(TransactionCreatedEvent), // 交易明细已创建
}

// 类型别名 - 每个事件都有唯一的载荷类型
pub type FuturesOrderCreatedEvent = NodeEvent<FuturesOrderCreatedPayload>;
pub type FuturesOrderCanceledEvent = NodeEvent<FuturesOrderCanceledPayload>;
pub type FuturesOrderFilledEvent = NodeEvent<FuturesOrderFilledPayload>;
pub type TakeProfitOrderCreatedEvent = NodeEvent<TakeProfitOrderCreatedPayload>;
pub type TakeProfitOrderFilledEvent = NodeEvent<TakeProfitOrderFilledPayload>;
pub type TakeProfitOrderCanceledEvent = NodeEvent<TakeProfitOrderCanceledPayload>;
pub type StopLossOrderCreatedEvent = NodeEvent<StopLossOrderCreatedPayload>;
pub type StopLossOrderFilledEvent = NodeEvent<StopLossOrderFilledPayload>;
pub type StopLossOrderCanceledEvent = NodeEvent<StopLossOrderCanceledPayload>;
pub type TransactionCreatedEvent = NodeEvent<TransactionCreatedPayload>;

// 载荷类型定义 - 每个事件都有唯一的载荷类型（避免From trait冲突）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderCreatedPayload {
    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,
}

impl FuturesOrderCreatedPayload {
    pub fn new(futures_order: VirtualOrder) -> Self {
        Self { futures_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderCanceledPayload {
    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,
}

impl FuturesOrderCanceledPayload {
    pub fn new(futures_order: VirtualOrder) -> Self {
        Self { futures_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderFilledPayload {
    #[serde(rename = "futuresOrder")]
    pub futures_order: VirtualOrder,
}

impl FuturesOrderFilledPayload {
    pub fn new(futures_order: VirtualOrder) -> Self {
        Self { futures_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitOrderCreatedPayload {
    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: VirtualOrder,
}

impl TakeProfitOrderCreatedPayload {
    pub fn new(take_profit_order: VirtualOrder) -> Self {
        Self { take_profit_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitOrderFilledPayload {
    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: VirtualOrder,
}

impl TakeProfitOrderFilledPayload {
    pub fn new(take_profit_order: VirtualOrder) -> Self {
        Self { take_profit_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeProfitOrderCanceledPayload {
    #[serde(rename = "takeProfitOrder")]
    pub take_profit_order: VirtualOrder,
}

impl TakeProfitOrderCanceledPayload {
    pub fn new(take_profit_order: VirtualOrder) -> Self {
        Self { take_profit_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrderCreatedPayload {
    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: VirtualOrder,
}

impl StopLossOrderCreatedPayload {
    pub fn new(stop_loss_order: VirtualOrder) -> Self {
        Self { stop_loss_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrderFilledPayload {
    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: VirtualOrder,
}

impl StopLossOrderFilledPayload {
    pub fn new(stop_loss_order: VirtualOrder) -> Self {
        Self { stop_loss_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrderCanceledPayload {
    #[serde(rename = "stopLossOrder")]
    pub stop_loss_order: VirtualOrder,
}

impl StopLossOrderCanceledPayload {
    pub fn new(stop_loss_order: VirtualOrder) -> Self {
        Self { stop_loss_order }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCreatedPayload {
    #[serde(rename = "transaction")]
    pub transaction: VirtualTransaction,
}

impl TransactionCreatedPayload {
    pub fn new(transaction: VirtualTransaction) -> Self {
        Self { transaction }
    }
}
