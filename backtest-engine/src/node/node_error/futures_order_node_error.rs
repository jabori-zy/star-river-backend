use std::sync::Arc;

use snafu::{Backtrace, Snafu};
use star_river_core::error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain};
use strategy_core::error::NodeError;
use virtual_trading::error::VirtualTradingSystemError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FuturesOrderNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    VirtualTradingSystem {
        source: VirtualTradingSystemError,
        backtrace: Backtrace,
    },

    #[snafu(display("cannot create order because current is processing order or unfilled order is not empty"))]
    CannotCreateOrder { backtrace: Backtrace },

    #[snafu(display("order config not found for order config id: {order_config_id}"))]
    OrderConfigNotFound { order_config_id: i32, backtrace: Backtrace },

    #[snafu(display("get symbol info failed for symbol: {symbol}"))]
    GetSymbolInfoFailed {
        symbol: String,
        source: Arc<dyn StarRiverErrorTrait + Send + Sync>,
        backtrace: Backtrace,
    },

    #[snafu(display("symbol info not found for symbol: {symbol}"))]
    SymbolInfoNotFound { symbol: String, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for FuturesOrderNodeError
impl StarRiverErrorTrait for FuturesOrderNodeError {
    fn get_prefix(&self) -> &'static str {
        "FUTURES_ORDER_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            FuturesOrderNodeError::NodeError { .. } => 1000,            // node error
            FuturesOrderNodeError::VirtualTradingSystem { .. } => 1001, //虚拟交易系统错误
            FuturesOrderNodeError::CannotCreateOrder { .. } => 1002,    //无法创建订单
            FuturesOrderNodeError::OrderConfigNotFound { .. } => 1003,  //订单配置未找到
            FuturesOrderNodeError::GetSymbolInfoFailed { .. } => 1004,  //获取交易对信息失败
            FuturesOrderNodeError::SymbolInfoNotFound { .. } => 1005,   //交易对信息未找到
        };

        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            FuturesOrderNodeError::NodeError { source, .. } => generate_error_code_chain(source),
            FuturesOrderNodeError::VirtualTradingSystem { source, .. } => generate_error_code_chain(source),
            FuturesOrderNodeError::CannotCreateOrder { .. } => vec![self.error_code()],
            FuturesOrderNodeError::OrderConfigNotFound { .. } => vec![self.error_code()],
            FuturesOrderNodeError::GetSymbolInfoFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            FuturesOrderNodeError::SymbolInfoNotFound { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                FuturesOrderNodeError::NodeError { source, .. } => source.error_message(language),
                FuturesOrderNodeError::VirtualTradingSystem { source, .. } => source.error_message(language),
                FuturesOrderNodeError::CannotCreateOrder { .. } => {
                    format!("无法创建订单，因为当前正在处理订单或未成交订单不为空")
                }
                FuturesOrderNodeError::OrderConfigNotFound { order_config_id, .. } => {
                    format!("订单配置未找到: {}", order_config_id)
                }
                FuturesOrderNodeError::GetSymbolInfoFailed { symbol, .. } => {
                    format!("获取交易对信息失败: {}", symbol)
                }
                FuturesOrderNodeError::SymbolInfoNotFound { symbol, .. } => {
                    format!("交易对信息未找到: {}", symbol)
                }
            },
        }
    }
}
