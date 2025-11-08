use star_river_core::error::{ErrorCode, StarRiverErrorTrait, ErrorLanguage};
use snafu::{Backtrace, Snafu};
use std::collections::HashMap;
use std::sync::Arc;
use virtual_trading::error::VirtualTradingSystemError;


#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FuturesOrderNodeError {
    #[snafu(transparent)]
    VirtualTradingSystem {
        source: VirtualTradingSystemError,
        backtrace: Backtrace,
    },

    #[snafu(display("cannot create order because current is processing order or unfilled order is not empty"))]
    CannotCreateOrder { backtrace: Backtrace },

    #[snafu(display("order config not found for input handle id: {input_handle_id}"))]
    OrderConfigNotFound { input_handle_id: String, backtrace: Backtrace },

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
            FuturesOrderNodeError::VirtualTradingSystem { source, .. } => source.error_code_chain(),
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
                FuturesOrderNodeError::VirtualTradingSystem { source, .. } => {
                    format!("虚拟交易系统错误，原因: {}", source)
                }
                FuturesOrderNodeError::CannotCreateOrder { .. } => {
                    format!("无法创建订单，因为当前正在处理订单或未成交订单不为空")
                }
                FuturesOrderNodeError::OrderConfigNotFound { input_handle_id, .. } => {
                    format!("订单配置未找到: {}", input_handle_id)
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
