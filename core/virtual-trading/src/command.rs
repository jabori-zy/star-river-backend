use std::{fmt::Debug, ops::Deref};

use chrono::{DateTime, Utc};
use derive_more::From;
use star_river_core::{
    custom_type::{NodeId, NodeName, OrderId, StrategyId},
    exchange::Exchange,
    order::{FuturesOrderSide, OrderType, TpslType},
};
use tokio::sync::oneshot;

use crate::error::VtsError;

// ================================ VTS Command Base ================================

#[derive(Debug)]
pub struct VtsCommandBase<S>
where
    S: Debug + Send + Sync + 'static,
{
    pub datetime: DateTime<Utc>,
    pub responder: oneshot::Sender<VtsResponse<S>>,
}

#[derive(Debug)]
pub struct GenericVtsCommand<T, S>
where
    T: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
    pub command_base: VtsCommandBase<S>,
    pub command_payload: T,
}

impl<T, S> GenericVtsCommand<T, S>
where
    T: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
    pub fn new(responder: oneshot::Sender<VtsResponse<S>>, command_payload: T) -> Self {
        let command_base = VtsCommandBase {
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload,
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.command_base.datetime
    }

    pub fn respond(self, response: VtsResponse<S>) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<T, S> Deref for GenericVtsCommand<T, S>
where
    T: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.command_payload
    }
}

// ================================ VTS Response ================================

#[derive(Debug)]
pub enum VtsResponse<P>
where
    P: Debug + Send + Sync + 'static,
{
    Success { payload: P, datetime: DateTime<Utc> },
    Fail { error: VtsError, datetime: DateTime<Utc> },
}

impl<P> VtsResponse<P>
where
    P: Debug + Send + Sync + 'static,
{
    /// Create success response
    pub fn success(payload: P) -> Self {
        Self::Success {
            payload,
            datetime: Utc::now(),
        }
    }

    /// Create fail response
    pub fn fail(error: VtsError) -> Self {
        Self::Fail {
            error,
            datetime: Utc::now(),
        }
    }

    /// Check if success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if fail
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    /// Get datetime
    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            Self::Success { datetime, .. } => *datetime,
            Self::Fail { datetime, .. } => *datetime,
        }
    }

    /// Get payload reference (if success)
    pub fn payload(&self) -> Option<&P> {
        match self {
            Self::Success { payload, .. } => Some(payload),
            Self::Fail { .. } => None,
        }
    }

    /// Get error reference (if fail)
    pub fn error(&self) -> Option<&VtsError> {
        match self {
            Self::Success { .. } => None,
            Self::Fail { error, .. } => Some(error),
        }
    }

    /// Consume self and return payload (if success)
    pub fn into_payload(self) -> Result<P, VtsError> {
        match self {
            Self::Success { payload, .. } => Ok(payload),
            Self::Fail { error, .. } => Err(error),
        }
    }

    /// Map VtsResponse<P> to VtsResponse<U>
    pub fn map<U, F>(self, f: F) -> VtsResponse<U>
    where
        U: Debug + Send + Sync + 'static,
        F: FnOnce(P) -> U,
    {
        match self {
            Self::Success { payload, datetime } => VtsResponse::Success {
                payload: f(payload),
                datetime,
            },
            Self::Fail { error, datetime } => VtsResponse::Fail { error, datetime },
        }
    }
}

// ================================ Specific Commands ================================

#[derive(Debug, From)]
pub enum VtsCommand {
    CreateOrder(CreateOrderCommand),
}

/// Type alias for CreateOrderCommand
pub type CreateOrderCommand = GenericVtsCommand<CreateOrderCmdPayload, CreateOrderRespPayload>;

/// Type alias for CreateOrderResponse
pub type CreateOrderResponse = VtsResponse<CreateOrderRespPayload>;

/// Create Order Command Payload
#[derive(Debug)]
pub struct CreateOrderCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub order_config_id: i32,
    pub symbol: String,
    pub exchange: Exchange,
    pub price: f64,
    pub order_side: FuturesOrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
    pub tp_type: Option<TpslType>,
    pub sl_type: Option<TpslType>,
    pub point: Option<f64>,
}

impl CreateOrderCmdPayload {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        order_config_id: i32,
        symbol: String,
        exchange: Exchange,
        price: f64,
        order_side: FuturesOrderSide,
        order_type: OrderType,
        quantity: f64,
        tp: Option<f64>,
        sl: Option<f64>,
        tp_type: Option<TpslType>,
        sl_type: Option<TpslType>,
        point: Option<f64>,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            node_name,
            order_config_id,
            symbol,
            exchange,
            price,
            order_side,
            order_type,
            quantity,
            tp,
            sl,
            tp_type,
            sl_type,
            point,
        }
    }
}

/// Create Order Response Payload
#[derive(Debug)]
pub struct CreateOrderRespPayload {
    pub order_id: OrderId,
}

impl CreateOrderRespPayload {
    pub fn new(order_id: OrderId) -> Self {
        Self { order_id }
    }
}
