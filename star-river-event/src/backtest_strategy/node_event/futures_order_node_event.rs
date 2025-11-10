use derive_more::From;
use serde::{Deserialize, Serialize};
use strategy_core::event::node::NodeEvent;
use strum::Display;
use virtual_trading::types::{VirtualOrder, VirtualTransaction};

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event_type")]
pub enum FuturesOrderNodeEvent {
    #[strum(serialize = "futures-order-created-event")]
    #[serde(rename = "futures-order-created-event")]
    FuturesOrderCreated(FuturesOrderCreatedEvent), // Order created

    #[strum(serialize = "futures-order-canceled-event")]
    #[serde(rename = "futures-order-canceled-event")]
    FuturesOrderCanceled(FuturesOrderCanceledEvent), // Order canceled

    #[strum(serialize = "futures-order-filled-event")]
    #[serde(rename = "futures-order-filled-event")]
    FuturesOrderFilled(FuturesOrderFilledEvent), // Order filled

    #[strum(serialize = "take-profit-order-created-event")]
    #[serde(rename = "take-profit-order-created-event")]
    TakeProfitOrderCreated(TakeProfitOrderCreatedEvent), // Take profit order created

    #[strum(serialize = "take-profit-order-filled-event")]
    #[serde(rename = "take-profit-order-filled-event")]
    TakeProfitOrderFilled(TakeProfitOrderFilledEvent), // Take profit order filled

    #[strum(serialize = "take-profit-order-canceled-event")]
    #[serde(rename = "take-profit-order-canceled-event")]
    TakeProfitOrderCanceled(TakeProfitOrderCanceledEvent), // Take profit order canceled

    #[strum(serialize = "stop-loss-order-created-event")]
    #[serde(rename = "stop-loss-order-created-event")]
    StopLossOrderCreated(StopLossOrderCreatedEvent), // Stop loss order created

    #[strum(serialize = "stop-loss-order-filled-event")]
    #[serde(rename = "stop-loss-order-filled-event")]
    StopLossOrderFilled(StopLossOrderFilledEvent), // Stop loss order filled

    #[strum(serialize = "stop-loss-order-canceled-event")]
    #[serde(rename = "stop-loss-order-canceled-event")]
    StopLossOrderCanceled(StopLossOrderCanceledEvent), // Stop loss order canceled

    #[strum(serialize = "transaction-created-event")]
    #[serde(rename = "transaction-created-event")]
    TransactionCreated(TransactionCreatedEvent), // Transaction created
}

// Type aliases - each event has a unique payload type
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

// Payload type definitions - each event has a unique payload type (to avoid From trait conflicts)
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
