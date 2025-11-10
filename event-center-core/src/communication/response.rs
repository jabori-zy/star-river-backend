use std::{fmt::Debug, sync::Arc};

use chrono::{DateTime, Utc};
use star_river_core::error::StarRiverErrorTrait;

// ================================ Engine Response ================================
#[derive(Debug)]
pub enum Response<P> {
    Success {
        payload: P,
        datetime: DateTime<Utc>,
    },
    Fail {
        error: Arc<dyn StarRiverErrorTrait>,
        datetime: DateTime<Utc>,
    },
}

impl<P> Response<P> {
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
    pub fn datetime(&self) -> DateTime<Utc> {
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

    /// 将Response<P>映射为Response<U>
    pub fn map<U, F>(self, f: F) -> Response<U>
    where
        F: FnOnce(P) -> U,
    {
        match self {
            Self::Success { payload, datetime } => Response::Success {
                payload: f(payload),
                datetime,
            },
            Self::Fail { error, datetime } => Response::Fail { error, datetime },
        }
    }
}
