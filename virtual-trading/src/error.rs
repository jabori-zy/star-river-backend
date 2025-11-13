use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::PositionId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode},
};

use crate::{command::VtsCommand, event::VtsEvent};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum VirtualTradingSystemError {
    #[snafu(display("order type [{order_type}] is unsupported"))]
    UnsupportedOrderType { order_type: String, backtrace: Backtrace },

    #[snafu(display("kline key not found for exchange [{exchange}] and symbol [{symbol}]"))]
    KlineKeyNotFound {
        exchange: String,
        symbol: String,
        backtrace: Backtrace,
    },

    #[snafu(display("event send failed: {source}"))]
    EventSendFailed {
        source: tokio::sync::broadcast::error::SendError<VtsEvent>,
        backtrace: Backtrace,
    },

    #[snafu(display("command send failed: {source}"))]
    CommandSendFailed {
        source: tokio::sync::mpsc::error::SendError<VtsCommand>,
        backtrace: Backtrace,
    },

    #[snafu(display("response recv failed: {source}"))]
    ResponseRecvFailed {
        source: tokio::sync::oneshot::error::RecvError,
        backtrace: Backtrace,
    },

    #[snafu(display("margin not enough, need: {need_margin}, available balance: {available_balance}"))]
    MarginNotEnough {
        need_margin: f64,
        available_balance: f64,
        backtrace: Backtrace,
    },

    #[snafu(display("order [{order_id}] not found."))]
    OrderNotFound { order_id: i32, backtrace: Backtrace },

    #[snafu(display("position [{position_id}] not found."))]
    PositionNotFound { position_id: PositionId, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl StarRiverErrorTrait for VirtualTradingSystemError {
    fn get_prefix(&self) -> &'static str {
        "VIRTUAL_TRADING_SYSTEM"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            VirtualTradingSystemError::UnsupportedOrderType { .. } => 1001, // unsupported order type
            VirtualTradingSystemError::KlineKeyNotFound { .. } => 1002,     // kline key not found
            VirtualTradingSystemError::EventSendFailed { .. } => 1003,      // event send failed
            VirtualTradingSystemError::MarginNotEnough { .. } => 1004,      // margin not enough
            VirtualTradingSystemError::OrderNotFound { .. } => 1005,        // order not found
            VirtualTradingSystemError::PositionNotFound { .. } => 1006,     // position not found
            VirtualTradingSystemError::CommandSendFailed { .. } => 1007,    // command send failed
            VirtualTradingSystemError::ResponseRecvFailed { .. } => 1008,   // response recv failed
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                VirtualTradingSystemError::UnsupportedOrderType { order_type, .. } => {
                    format!("不支持的订单类型 [{}]", order_type)
                }
                VirtualTradingSystemError::KlineKeyNotFound { exchange, symbol, .. } => {
                    format!("k线缓存key未找到 for exchange [{}] and symbol [{}]", exchange, symbol)
                }
                VirtualTradingSystemError::EventSendFailed { source, .. } => {
                    format!("事件发送失败: {}", source)
                }
                VirtualTradingSystemError::MarginNotEnough {
                    need_margin,
                    available_balance,
                    ..
                } => {
                    format!("保证金不足, 需要: {need_margin}, 当前可用余额: {available_balance}")
                }
                VirtualTradingSystemError::OrderNotFound { order_id, .. } => {
                    format!("订单 {} 未找到.", order_id)
                }
                VirtualTradingSystemError::PositionNotFound { position_id, .. } => {
                    format!("仓位 {} 未找到.", position_id)
                }
                VirtualTradingSystemError::CommandSendFailed { source, .. } => {
                    format!("命令发送失败: {}", source)
                }
                VirtualTradingSystemError::ResponseRecvFailed { source, .. } => {
                    format!("响应接收失败: {}", source)
                }
            },
        }
    }

    fn http_status_code(&self) -> star_river_core::error::StatusCode {
        match self {
            VirtualTradingSystemError::UnsupportedOrderType { .. } => StatusCode::BAD_REQUEST,
            VirtualTradingSystemError::KlineKeyNotFound { .. } | VirtualTradingSystemError::EventSendFailed { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            VirtualTradingSystemError::MarginNotEnough { .. } => StatusCode::BAD_REQUEST,
            VirtualTradingSystemError::OrderNotFound { .. } => StatusCode::NOT_FOUND,
            VirtualTradingSystemError::PositionNotFound { .. } => StatusCode::NOT_FOUND,
            VirtualTradingSystemError::CommandSendFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            VirtualTradingSystemError::ResponseRecvFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            VirtualTradingSystemError::UnsupportedOrderType { .. }
            | VirtualTradingSystemError::KlineKeyNotFound { .. }
            | VirtualTradingSystemError::EventSendFailed { .. } => {
                vec![self.error_code()]
            }
            VirtualTradingSystemError::MarginNotEnough { .. } => vec![self.error_code()],
            VirtualTradingSystemError::OrderNotFound { .. } => vec![self.error_code()],
            VirtualTradingSystemError::PositionNotFound { .. } => vec![self.error_code()],
            VirtualTradingSystemError::CommandSendFailed { .. } => vec![self.error_code()],
            VirtualTradingSystemError::ResponseRecvFailed { .. } => vec![self.error_code()],
        }
    }
}
