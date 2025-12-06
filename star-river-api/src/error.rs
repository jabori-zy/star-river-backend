use snafu::{Backtrace, Snafu};
use star_river_core::error::{
    ErrorCode, StatusCode,
    error_trait::{ErrorLanguage, StarRiverErrorTrait},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum ApiError {
    #[snafu(display("parse datetime failed: {datetime}. reason: {source}"))]
    ParseDataTimeFailed {
        datetime: String,
        source: chrono::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Invalid timestamp: {timestamp}"))]
    InvalidTimestamp { timestamp: i64, backtrace: Backtrace },

    #[snafu(display("Transform timestamp failed: {timestamp}"))]
    TransformTimestampFailed { timestamp: i64, backtrace: Backtrace },

    #[snafu(display("{name} length is {length}, exceeds the maximum limit {max_length}"))]
    CharacterLengthExceedsLimit {
        name: String,
        length: i32,
        max_length: i32,
        backtrace: Backtrace,
    },

    #[snafu(display("{name} is empty"))]
    EmptyCharacter { name: String, backtrace: Backtrace },

    #[snafu(display("page must be greater than one. requested page: {page}"))]
    PageMustGreaterThanOne { page: u64, backtrace: Backtrace },

    #[snafu(display("items per page must be between 1 and 100. requested items per page: {items_per_page}"))]
    TooManyItemsPerPage { items_per_page: u64, backtrace: Backtrace },

    #[snafu(display("deserialize params failed. reason: {source}"))]
    DeserializeParamsFailed { source: serde_json::Error, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for ApiError {
    fn get_prefix(&self) -> &'static str {
        "Api"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            ApiError::ParseDataTimeFailed { .. } => 1001,         // Parse time failed
            ApiError::InvalidTimestamp { .. } => 1002,            // Invalid timestamp
            ApiError::TransformTimestampFailed { .. } => 1003,    // Timestamp transformation failed
            ApiError::CharacterLengthExceedsLimit { .. } => 1004, // Character length exceeds maximum limit
            ApiError::EmptyCharacter { .. } => 1005,              // Character is empty
            ApiError::PageMustGreaterThanOne { .. } => 1006,      // Page number must be greater than or equal to 1
            ApiError::TooManyItemsPerPage { .. } => 1007,         // Items per page must be between 1 and 100
            ApiError::DeserializeParamsFailed { .. } => 1008,     // Deserialize parameters failed
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                ApiError::ParseDataTimeFailed { source, .. } => {
                    format!("解析时间失败，原因: {}", source)
                }
                ApiError::InvalidTimestamp { timestamp, .. } => {
                    format!("无效的时间戳: {}", timestamp)
                }
                ApiError::TransformTimestampFailed { timestamp, .. } => {
                    format!("时间戳转换失败: {}", timestamp)
                }
                ApiError::CharacterLengthExceedsLimit {
                    name, length, max_length, ..
                } => {
                    format!("{name} 长度为 {length}，超过最大限制 {max_length}")
                }
                ApiError::EmptyCharacter { name, .. } => {
                    format!("{name} 不能为空")
                }
                ApiError::PageMustGreaterThanOne { page, .. } => {
                    format!("页码必须大于等于1。请求页码: {}", page)
                }
                ApiError::TooManyItemsPerPage { items_per_page, .. } => {
                    format!("每页数量必须大于等于1且小于等于100。请求每页数量: {}", items_per_page)
                }
                ApiError::DeserializeParamsFailed { source, .. } => {
                    format!("解析参数失败，原因: {}", source)
                }
            },
        }
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            ApiError::ParseDataTimeFailed { .. }
            | ApiError::InvalidTimestamp { .. }
            | ApiError::CharacterLengthExceedsLimit { .. }
            | ApiError::EmptyCharacter { .. }
            | ApiError::PageMustGreaterThanOne { .. }
            | ApiError::TooManyItemsPerPage { .. } => StatusCode::BAD_REQUEST,
            ApiError::TransformTimestampFailed { .. } => StatusCode::BAD_REQUEST,
            ApiError::DeserializeParamsFailed { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            ApiError::ParseDataTimeFailed { .. }
            | ApiError::InvalidTimestamp { .. }
            | ApiError::CharacterLengthExceedsLimit { .. }
            | ApiError::EmptyCharacter { .. }
            | ApiError::TransformTimestampFailed { .. }
            | ApiError::PageMustGreaterThanOne { .. }
            | ApiError::TooManyItemsPerPage { .. }
            | ApiError::DeserializeParamsFailed { .. } => vec![self.error_code()],
        }
    }
}
