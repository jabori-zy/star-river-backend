use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use crate::error::error_trait::StarRiverErrorTrait;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CacheEngineError {
    #[snafu(display("key [{key}] not found"))]
    KeyNotFound { key: String, backtrace: Backtrace },
}

impl StarRiverErrorTrait for CacheEngineError {
    fn get_prefix(&self) -> &'static str {
        "CACHE_ENGINE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            CacheEngineError::KeyNotFound { .. } => 1001, // 数据长度小于lookback
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> std::collections::HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(self, CacheEngineError::KeyNotFound { .. })
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            CacheEngineError::KeyNotFound { .. } => vec![self.error_code()],
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                CacheEngineError::KeyNotFound { key, .. } => {
                    format!("缓存key [{}] 不存在", key)
                }
            },
        }
    }
}
