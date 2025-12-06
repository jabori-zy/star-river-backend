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

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if response is failure
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }

    /// Get timestamp
    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            Self::Success { datetime, .. } => *datetime,
            Self::Fail { datetime, .. } => *datetime,
        }
    }

    /// Get reference to payload (if successful)
    pub fn payload(&self) -> Option<&P> {
        match self {
            Self::Success { payload, .. } => Some(payload),
            Self::Fail { .. } => None,
        }
    }

    /// Get reference to error (if failed)
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

    /// Map Response<P> to Response<U>
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
