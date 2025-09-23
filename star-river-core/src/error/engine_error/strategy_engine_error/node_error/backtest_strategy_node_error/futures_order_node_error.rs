use crate::error::ErrorCode;
use crate::error::error_trait::Language;
use crate::error::virtual_trading_system_error::VirtualTradingSystemError;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FuturesOrderNodeError {
    #[snafu(display("futures order node backtest config field value is null: {field_name}"))]
    ConfigFieldValueNull { field_name: String, backtrace: Backtrace },

    #[snafu(display("futures order node backtest config deserialization failed. reason: {source}"))]
    ConfigDeserializationFailed {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(transparent)]
    VirtualTradingSystem {
        source: VirtualTradingSystemError,
        backtrace: Backtrace,
    },

    #[snafu(display("cannot create order because current is processing order or unfilled order is not empty"))]
    CannotCreateOrder { backtrace: Backtrace },

    #[snafu(display("order config not found for input handle id: {input_handle_id}"))]
    OrderConfigNotFound {
        input_handle_id: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for FuturesOrderNodeError
impl crate::error::error_trait::StarRiverErrorTrait for FuturesOrderNodeError {
    fn get_prefix(&self) -> &'static str {
        "FUTURES_ORDER_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            // HTTP and JSON errors (1001-1004)
            FuturesOrderNodeError::ConfigFieldValueNull { .. } => 1001,
            FuturesOrderNodeError::ConfigDeserializationFailed { .. } => 1002,
            FuturesOrderNodeError::VirtualTradingSystem { .. } => 1003,
            FuturesOrderNodeError::CannotCreateOrder { .. } => 1004,
            FuturesOrderNodeError::OrderConfigNotFound { .. } => 1005,
        };

        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        HashMap::new()
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            FuturesOrderNodeError::ConfigFieldValueNull { .. }
                | FuturesOrderNodeError::ConfigDeserializationFailed { .. }
                | FuturesOrderNodeError::CannotCreateOrder { .. }
                | FuturesOrderNodeError::OrderConfigNotFound { .. }
        )
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        // All FuturesOrderNodeError variants either have no source or
        // have external sources (serde_json::Error) that don't implement our trait
        match self {
            FuturesOrderNodeError::ConfigFieldValueNull { .. }
            | FuturesOrderNodeError::ConfigDeserializationFailed { .. } => vec![self.error_code()],
            FuturesOrderNodeError::VirtualTradingSystem { source, .. } => source.error_code_chain(),
            FuturesOrderNodeError::CannotCreateOrder { .. } => vec![self.error_code()],
            FuturesOrderNodeError::OrderConfigNotFound { .. } => vec![self.error_code()],
        }
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                FuturesOrderNodeError::ConfigFieldValueNull { field_name, .. } => {
                    format!("期货订单节点回测配置字段值为空: {}", field_name)
                }
                FuturesOrderNodeError::ConfigDeserializationFailed { source, .. } => {
                    format!("期货订单节点回测配置反序列化失败，原因: {}", source)
                }
                FuturesOrderNodeError::VirtualTradingSystem { source, .. } => {
                    format!("虚拟交易系统错误，原因: {}", source)
                }
                FuturesOrderNodeError::CannotCreateOrder { .. } => {
                    format!("无法创建订单，因为当前正在处理订单或未成交订单不为空")
                }
                FuturesOrderNodeError::OrderConfigNotFound { input_handle_id, .. } => {
                    format!("订单配置未找到: {}", input_handle_id)
                }
            },
        }
    }
}
