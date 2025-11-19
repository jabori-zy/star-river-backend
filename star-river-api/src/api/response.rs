use std::error::Error;

use chrono::Utc;
use serde::Serialize;
use snafu::Report;
use star_river_core::error::{ErrorCode, error_trait::StarRiverErrorTrait};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize, ToSchema)]
pub struct NewApiResponse<T> {
    pub success: bool,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<ErrorCode>>,
}

impl<T> NewApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            timestamp: Utc::now().to_string(),
            data: Some(data),
            message: None,
            error_code: None,
            error_code_chain: None,
        }
    }

    pub fn error(error: impl Error + StarRiverErrorTrait) -> Self {
        let report = Report::from_error(&error);
        Self {
            success: false,
            timestamp: Utc::now().to_string(),
            data: None,
            error_code: Some(error.error_code()),
            error_code_chain: Some(error.error_code_chain()),
            message: Some(report.to_string()),
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(untagged)]
pub enum ApiResponseEnum<T> {
    Success {
        success: bool,
        timestamp: String,
        data: T,
    },
    Error {
        success: bool,
        timestamp: String,
        message: String,
        #[serde(rename = "errorCode")]
        error_code: ErrorCode,
        #[serde(rename = "errorCodeChain")]
        error_code_chain: Vec<ErrorCode>,
    },
}

impl<T> ApiResponseEnum<T> {
    pub fn success(data: T) -> Self {
        Self::Success {
            success: true,
            timestamp: Utc::now().to_string(),
            data,
        }
    }

    pub fn error(error: impl Error + StarRiverErrorTrait) -> Self {
        let report = Report::from_error(&error);
        Self::Error {
            success: false,
            timestamp: Utc::now().to_string(),
            message: report.to_string().trim().to_string(),
            error_code: error.error_code(),
            error_code_chain: error.error_code_chain(),
        }
    }
}
