
use chrono::Utc;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::oneshot;
use star_river_core::custom_type::NodeId;
use super::{StrategyCommandTrait, StrategyResponseTrait};
use std::fmt::Debug;

// ================================ Strategy Command ================================
#[derive(Debug)]
pub struct StrategyCommandBase<S> {
    pub node_id: NodeId,
    pub datetime: chrono::DateTime<Utc>,
    pub responder: oneshot::Sender<StrategyResponse<S>>,
}

#[derive(Debug)]
pub struct StrategyCommand<P, S>
{
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
        &self
            .command_payload
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
    /// 创建成功响应
    pub fn success(payload: P) -> Self {
        Self::Success {
            payload,
            datetime: Utc::now(),
        }
    }

    /// 创建失败响应
    pub fn fail(error: Arc<dyn StarRiverErrorTrait>) -> Self {
        Self::Fail {
            error,
            datetime: Utc::now(),
        }
    }

    /// 判断是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// 判断是否失败
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    /// 获取时间戳
    pub fn datetime(&self) -> chrono::DateTime<Utc> {
        match self {
            Self::Success { datetime, .. } => *datetime,
            Self::Fail { datetime, .. } => *datetime,
        }
    }

    /// 获取payload的引用（如果成功）
    pub fn payload(&self) -> Option<&P> {
        match self {
            Self::Success { payload, .. } => Some(payload),
            Self::Fail { .. } => None,
        }
    }

    /// 获取error的引用（如果失败）
    pub fn error(&self) -> Option<&Arc<dyn StarRiverErrorTrait>> {
        match self {
            Self::Success { .. } => None,
            Self::Fail { error, .. } => Some(error),
        }
    }

    /// 消费self并返回payload（如果成功）
    pub fn into_payload(self) -> Result<P, Arc<dyn StarRiverErrorTrait>> {
        match self {
            Self::Success { payload, .. } => Ok(payload),
            Self::Fail { error, .. } => Err(error),
        }
    }

    /// 将StrategyResponse<P>映射为StrategyResponse<U>
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

impl<S> StrategyResponseTrait for StrategyResponse<S> 
where
    S: Debug + Send + Sync + 'static,
{
}