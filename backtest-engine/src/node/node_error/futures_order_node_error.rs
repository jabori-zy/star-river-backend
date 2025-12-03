use std::sync::Arc;

use event_center::EventCenterError;
use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::NodeName,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, generate_error_code_chain},
};
use strategy_core::error::{NodeError, NodeStateMachineError};
use virtual_trading::error::VtsError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FuturesOrderNodeError {
    #[snafu(transparent)]
    NodeError { source: NodeError, backtrace: Backtrace },

    #[snafu(transparent)]
    VirtualTradingSystem { source: VtsError, backtrace: Backtrace },

    #[snafu(transparent)]
    EventCenterError { source: EventCenterError, backtrace: Backtrace },

    #[snafu(transparent)]
    NodeStateMachineError {
        source: NodeStateMachineError,
        backtrace: Backtrace,
    },

    #[snafu(display("@[{node_name}] config {:?} is processing order, skip", order_config_id))]
    CannotCreateOrder {
        node_name: NodeName,
        order_config_id: i32,
        backtrace: Backtrace,
    },

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

    #[snafu(display("@[{node_name}] exchange mode not configured"))]
    ExchangeModeNotConfigured { node_name: NodeName, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for FuturesOrderNodeError
impl StarRiverErrorTrait for FuturesOrderNodeError {
    fn get_prefix(&self) -> &'static str {
        "FUTURES_ORDER_NODE"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            FuturesOrderNodeError::NodeError { .. } => 1000,                 // node error
            FuturesOrderNodeError::VirtualTradingSystem { .. } => 1001,      //虚拟交易系统错误
            FuturesOrderNodeError::NodeStateMachineError { .. } => 1002,     //节点状态机错误
            FuturesOrderNodeError::CannotCreateOrder { .. } => 1003,         //无法创建订单
            FuturesOrderNodeError::EventCenterError { .. } => 1004,          //事件中心错误
            FuturesOrderNodeError::OrderConfigNotFound { .. } => 1005,       //订单配置未找到
            FuturesOrderNodeError::GetSymbolInfoFailed { .. } => 1006,       //获取交易对信息失败
            FuturesOrderNodeError::SymbolInfoNotFound { .. } => 1007,        //交易对信息未找到
            FuturesOrderNodeError::ExchangeModeNotConfigured { .. } => 1008, //交易所模式未配置
        };

        format!("{}_{:04}", prefix, code)
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            FuturesOrderNodeError::NodeError { source, .. } => generate_error_code_chain(source, self.error_code()),
            FuturesOrderNodeError::VirtualTradingSystem { source, .. } => generate_error_code_chain(source, self.error_code()),
            FuturesOrderNodeError::EventCenterError { source, .. } => generate_error_code_chain(source, self.error_code()),
            FuturesOrderNodeError::NodeStateMachineError { source, .. } => generate_error_code_chain(source, self.error_code()),
            FuturesOrderNodeError::CannotCreateOrder { .. } => vec![self.error_code()],
            FuturesOrderNodeError::OrderConfigNotFound { .. } => vec![self.error_code()],
            FuturesOrderNodeError::GetSymbolInfoFailed { source, .. } => {
                let mut chain = source.error_code_chain();
                chain.push(self.error_code());
                chain
            }
            FuturesOrderNodeError::SymbolInfoNotFound { .. } => vec![self.error_code()],
            FuturesOrderNodeError::ExchangeModeNotConfigured { .. } => vec![self.error_code()],
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                FuturesOrderNodeError::NodeError { source, .. } => source.error_message(language),
                FuturesOrderNodeError::VirtualTradingSystem { source, .. } => source.error_message(language),
                FuturesOrderNodeError::EventCenterError { source, .. } => source.error_message(language),
                FuturesOrderNodeError::NodeStateMachineError { source, .. } => source.error_message(language),
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
                FuturesOrderNodeError::ExchangeModeNotConfigured { node_name, .. } => {
                    format!("@[{node_name}] 交易所模式未配置")
                }
            },
        }
    }
}
