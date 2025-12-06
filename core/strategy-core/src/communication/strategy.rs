use std::{fmt::Debug, ops::Deref, sync::Arc};

use chrono::Utc;
use star_river_core::{custom_type::NodeId, error::error_trait::StarRiverErrorTrait};
use tokio::sync::oneshot;

use super::{StrategyCommandTrait, StrategyResponseTrait};

// ================================ Strategy Command ================================
#[derive(Debug)]
pub struct StrategyCommandBase<S> {
    pub node_id: NodeId,
    pub datetime: chrono::DateTime<Utc>,
    pub responder: oneshot::Sender<StrategyResponse<S>>,
}

#[derive(Debug)]
pub struct StrategyCommand<P, S> {
    pub command_base: StrategyCommandBase<S>,
    pub command_payload: P,
}

impl<P, S> StrategyCommand<P, S> {
    pub fn new(node_id: NodeId, responder: oneshot::Sender<StrategyResponse<S>>, command_payload: P) -> Self {
        let command_base = StrategyCommandBase {
            node_id,
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.command_base.node_id.clone()
    }

    pub fn datetime(&self) -> chrono::DateTime<Utc> {
        self.command_base.datetime
    }

    pub fn respond(self, response: StrategyResponse<S>) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<P, S> StrategyCommandTrait for StrategyCommand<P, S>
where
    P: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
}

impl<P, S> Deref for StrategyCommand<P, S> {
    type Target = P;
    fn deref(&self) -> &Self::Target {
        &self.command_payload
    }
}

// ================================ Strategy Response ================================

#[derive(Debug)]
pub enum StrategyResponse<P> {
    Success {
        payload: P,
        datetime: chrono::DateTime<Utc>,
    },
    Fail {
        error: Arc<dyn StarRiverErrorTrait>,
        datetime: chrono::DateTime<Utc>,
    },
}

impl<P> StrategyResponse<P> {
    /// Create success response
    pub fn success(payload: P) -> Self {
        Self::Success {
            payload,
            datetime: Utc::now(),
        }
    }

    /// Create failure response
    pub fn fail(error: Arc<dyn StarRiverErrorTrait>) -> Self {
        Self::Fail {
            error,
            datetime: Utc::now(),
        }
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if failed
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    /// Get timestamp
    pub fn datetime(&self) -> chrono::DateTime<Utc> {
        match self {
            Self::Success { datetime, .. } => *datetime,
            Self::Fail { datetime, .. } => *datetime,
        }
    }

    /// Get payload reference (if successful)
    pub fn payload(&self) -> Option<&P> {
        match self {
            Self::Success { payload, .. } => Some(payload),
            Self::Fail { .. } => None,
        }
    }

    /// Get error reference (if failed)
    pub fn error(&self) -> Option<&Arc<dyn StarRiverErrorTrait>> {
        match self {
            Self::Success { .. } => None,
            Self::Fail { error, .. } => Some(error),
        }
    }

    /// Consume self and return payload (if successful)
    pub fn into_payload(self) -> Result<P, Arc<dyn StarRiverErrorTrait>> {
        match self {
            Self::Success { payload, .. } => Ok(payload),
            Self::Fail { error, .. } => Err(error),
        }
    }

    /// Map StrategyResponse<P> to StrategyResponse<U>
    pub fn map<U, F>(self, f: F) -> StrategyResponse<U>
    where
        F: FnOnce(P) -> U,
    {
        match self {
            Self::Success { payload, datetime } => StrategyResponse::Success {
                payload: f(payload),
                datetime,
            },
            Self::Fail { error, datetime } => StrategyResponse::Fail { error, datetime },
        }
    }
}

impl<S> StrategyResponseTrait for StrategyResponse<S> where S: Debug + Send + Sync + 'static {}
