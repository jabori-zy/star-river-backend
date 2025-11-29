use snafu::{Backtrace, Snafu};
use star_river_core::{
    custom_type::PositionId,
    error::{ErrorCode, ErrorLanguage, StarRiverErrorTrait, StatusCode},
};

use crate::{command::VtsCommand, event::VtsEvent};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum VtsError {
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

    #[snafu(display("only support one direction, the order side is [{order_side}] but the position side is [{position_side}]"))]
    OnlyOneDirectionSupported {
        order_side: String,
        position_side: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Position #{}: the tp order quantity is more than the position quantity, close all position, tp order quantity: [{tp_order_quantity}], position quantity: [{pos_quantity}]"
    ))]
    TpOrderQuantityMoreThanPosQuantity {
        position_id: PositionId,
        tp_order_quantity: f64,
        pos_quantity: f64,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Position #{}: the sl order quantity is more than the position quantity, close all position, sl order quantity: [{sl_order_quantity}], position quantity: [{pos_quantity}]"
    ))]
    SlOrderQuantityMoreThanPosQuantity {
        position_id: PositionId,
        sl_order_quantity: f64,
        pos_quantity: f64,
        backtrace: Backtrace,
    },
}

// Implement the StarRiverErrorTrait for IndicatorError
impl StarRiverErrorTrait for VtsError {
    fn get_prefix(&self) -> &'static str {
        "VIRTUAL_TRADING_SYSTEM"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            VtsError::UnsupportedOrderType { .. } => 1001,               // unsupported order type
            VtsError::KlineKeyNotFound { .. } => 1002,                   // kline key not found
            VtsError::EventSendFailed { .. } => 1003,                    // event send failed
            VtsError::MarginNotEnough { .. } => 1004,                    // margin not enough
            VtsError::OrderNotFound { .. } => 1005,                      // order not found
            VtsError::PositionNotFound { .. } => 1006,                   // position not found
            VtsError::CommandSendFailed { .. } => 1007,                  // command send failed
            VtsError::ResponseRecvFailed { .. } => 1008,                 // response recv failed
            VtsError::OnlyOneDirectionSupported { .. } => 1009,          // only one direction supported
            VtsError::TpOrderQuantityMoreThanPosQuantity { .. } => 1010, // tp order quantity more than pos quantity
            VtsError::SlOrderQuantityMoreThanPosQuantity { .. } => 1011, // sl order quantity more than pos quantity
        };
        format!("{}_{:04}", prefix, code)
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                VtsError::UnsupportedOrderType { order_type, .. } => {
                    format!("不支持的订单类型 [{}]", order_type)
                }
                VtsError::KlineKeyNotFound { exchange, symbol, .. } => {
                    format!("k线缓存key未找到 for exchange [{}] and symbol [{}]", exchange, symbol)
                }
                VtsError::EventSendFailed { source, .. } => {
                    format!("事件发送失败: {}", source)
                }
                VtsError::MarginNotEnough {
                    need_margin,
                    available_balance,
                    ..
                } => {
                    format!("保证金不足, 需要: {need_margin}, 当前可用余额: {available_balance}")
                }
                VtsError::OrderNotFound { order_id, .. } => {
                    format!("订单 {} 未找到.", order_id)
                }
                VtsError::PositionNotFound { position_id, .. } => {
                    format!("仓位 {} 未找到.", position_id)
                }
                VtsError::CommandSendFailed { source, .. } => {
                    format!("命令发送失败: {}", source)
                }
                VtsError::ResponseRecvFailed { source, .. } => {
                    format!("响应接收失败: {}", source)
                }
                VtsError::OnlyOneDirectionSupported {
                    order_side, position_side, ..
                } => {
                    format!("只支持单向持仓, 订单方向是 [{order_side}] 但仓位方向是 [{position_side}]")
                }
                VtsError::TpOrderQuantityMoreThanPosQuantity {
                    position_id,
                    tp_order_quantity,
                    pos_quantity,
                    ..
                } => {
                    format!(
                        "仓位 #{}: 止盈订单数量大于仓位数量, 全部平仓. 止盈订单数量: [{tp_order_quantity}], 持仓数量: [{pos_quantity}]",
                        position_id
                    )
                }
                VtsError::SlOrderQuantityMoreThanPosQuantity {
                    position_id,
                    sl_order_quantity,
                    pos_quantity,
                    ..
                } => {
                    format!(
                        "仓位 #{}: 止损订单数量大于仓位数量, 全部平仓. 止损订单数量: [{sl_order_quantity}], 持仓数量: [{pos_quantity}]",
                        position_id
                    )
                }
            },
        }
    }

    fn http_status_code(&self) -> star_river_core::error::StatusCode {
        match self {
            VtsError::UnsupportedOrderType { .. } => StatusCode::BAD_REQUEST,
            VtsError::KlineKeyNotFound { .. } | VtsError::EventSendFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            VtsError::MarginNotEnough { .. } => StatusCode::BAD_REQUEST,
            VtsError::OrderNotFound { .. } => StatusCode::NOT_FOUND,
            VtsError::PositionNotFound { .. } => StatusCode::NOT_FOUND,
            VtsError::CommandSendFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            VtsError::ResponseRecvFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            VtsError::OnlyOneDirectionSupported { .. } => StatusCode::BAD_REQUEST,
            VtsError::TpOrderQuantityMoreThanPosQuantity { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            VtsError::SlOrderQuantityMoreThanPosQuantity { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            VtsError::UnsupportedOrderType { .. }
            | VtsError::KlineKeyNotFound { .. }
            | VtsError::EventSendFailed { .. }
            | VtsError::MarginNotEnough { .. }
            | VtsError::OrderNotFound { .. }
            | VtsError::PositionNotFound { .. }
            | VtsError::CommandSendFailed { .. }
            | VtsError::ResponseRecvFailed { .. }
            | VtsError::OnlyOneDirectionSupported { .. }
            | VtsError::TpOrderQuantityMoreThanPosQuantity { .. }
            | VtsError::SlOrderQuantityMoreThanPosQuantity { .. } => vec![self.error_code()],
        }
    }
}
