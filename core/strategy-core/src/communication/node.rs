use std::{fmt::Debug, ops::Deref, sync::Arc};

use chrono::Utc;
use star_river_core::{
    custom_type::{NodeId, NodeName},
    error::error_trait::StarRiverErrorTrait,
};
use tokio::sync::oneshot;

// ================================ Node Command ================================
#[derive(Debug)]
pub struct NodeCommandBase<S>
where
    S: Debug + Send + Sync + 'static,
{
    pub node_id: NodeId,
    pub node_name: NodeName,
    pub datetime: chrono::DateTime<Utc>,
    pub responder: oneshot::Sender<NodeResponse<S>>,
}

#[derive(Debug)]
pub struct NodeCommand<T, S>
where
    T: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
    pub command_base: NodeCommandBase<S>,
    pub command_payload: T,
}

impl<T, S> NodeCommand<T, S>
where
    T: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
    pub fn new(node_id: NodeId, node_name: NodeName, responder: oneshot::Sender<NodeResponse<S>>, command_payload: T) -> Self {
        let command_base = NodeCommandBase {
            node_id,
            node_name,
            datetime: Utc::now(),
            responder,
        };
        Self {
            command_base,
            command_payload,
        }
    }

    pub fn node_id(&self) -> &NodeId {
        &self.command_base.node_id
    }

    pub fn node_name(&self) -> &NodeName {
        &self.command_base.node_name
    }

    pub fn datetime(&self) -> chrono::DateTime<Utc> {
        self.command_base.datetime
    }

    pub fn respond(self, response: NodeResponse<S>) {
        let _ = self.command_base.responder.send(response);
    }
}

impl<T, S> Deref for NodeCommand<T, S>
where
    T: Debug + Send + Sync + 'static,
    S: Debug + Send + Sync + 'static,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.command_payload
    }
}

// ================================ Node Response ================================

#[derive(Debug)]
pub enum NodeResponse<P>
where
    P: Debug + Send + Sync + 'static,
{
    Success {
        node_id: NodeId,
        node_name: NodeName,
        payload: P,
        datetime: chrono::DateTime<Utc>,
    },
    Fail {
        node_id: NodeId,
        node_name: NodeName,
        error: Arc<dyn StarRiverErrorTrait>,
        datetime: chrono::DateTime<Utc>,
    },
}

impl<P> NodeResponse<P>
where
    P: Debug + Send + Sync + 'static,
{
    /// Create success response
    pub fn success(node_id: NodeId, node_name: NodeName, payload: P) -> Self {
        Self::Success {
            node_id,
            node_name,
            payload,
            datetime: Utc::now(),
        }
    }

    /// Create failure response
    pub fn fail(node_id: NodeId, node_name: NodeName, error: Arc<dyn StarRiverErrorTrait>) -> Self {
        Self::Fail {
            node_id,
            node_name,
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

    /// Get node ID
    pub fn node_id(&self) -> NodeId {
        match self {
            Self::Success { node_id, .. } => node_id.clone(),
            Self::Fail { node_id, .. } => node_id.clone(),
        }
    }
    /// Get node name
    pub fn node_name(&self) -> NodeName {
        match self {
            Self::Success { node_name, .. } => node_name.clone(),
            Self::Fail { node_name, .. } => node_name.clone(),
        }
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

    /// Map NodeResponse<P> to NodeResponse<U>
    pub fn map<U, F>(self, f: F) -> NodeResponse<U>
    where
        U: Debug + Send + Sync + 'static,
        F: FnOnce(P) -> U,
    {
        match self {
            Self::Success {
                node_id,
                node_name,
                payload,
                datetime,
            } => NodeResponse::Success {
                node_id,
                node_name,
                payload: f(payload),
                datetime,
            },
            Self::Fail {
                node_id,
                node_name,
                error,
                datetime,
            } => NodeResponse::Fail {
                node_id,
                node_name,
                error,
                datetime,
            },
        }
    }
}
