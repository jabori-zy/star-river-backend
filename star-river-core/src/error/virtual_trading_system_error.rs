use crate::error::error_trait::Language;
use crate::error::ErrorCode;
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum VirtualTradingSystemError {
    #[snafu(display("order type [{order_type}] is unsupported"))]
    UnsupportedOrderType {
        order_type: String,
        backtrace: Backtrace,
    },

    #[snafu(display("kline key not found for exchange [{exchange}] and symbol [{symbol}]"))]
    KlineKeyNotFound {
        exchange: String,
        symbol: String,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl crate::error::error_trait::StarRiverErrorTrait for VirtualTradingSystemError {
    fn get_prefix(&self) -> &'static str {
        "VIRTUAL_TRADING_SYSTEM"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            VirtualTradingSystemError::UnsupportedOrderType { .. } => 1001,
            VirtualTradingSystemError::KlineKeyNotFound { .. } => 1002,
        };
        format!("{}_{:04}", prefix, code)
    }

    fn context(&self) -> HashMap<&'static str, String> {
        let ctx = HashMap::new();
        ctx
    }

    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            VirtualTradingSystemError::UnsupportedOrderType { .. } |
            VirtualTradingSystemError::KlineKeyNotFound { .. }
        )
    }

    fn get_error_message(&self, language: Language) -> String {
        match language {
            Language::English => self.to_string(),
            Language::Chinese => match self {
                VirtualTradingSystemError::UnsupportedOrderType { order_type, .. } => {
                    format!("不支持的订单类型 [{}]", order_type)
                }
                VirtualTradingSystemError::KlineKeyNotFound { exchange, symbol, .. } => {
                    format!("k线缓存key未找到 for exchange [{}] and symbol [{}]", exchange, symbol)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            // CreateIndicatorFailed has source but serde_json::Error doesn't implement our trait
            // So we start the chain here
            VirtualTradingSystemError::UnsupportedOrderType { .. } |
            VirtualTradingSystemError::KlineKeyNotFound { .. } => vec![self.error_code()],
        }
    }
}
