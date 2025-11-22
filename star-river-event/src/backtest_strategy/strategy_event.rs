use chrono::Utc;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::{custom_type::StrategyId, system::DateTimeUtc};
// use log_event::{NodeStateLogEvent, StrategyRunningLogEvent, LogLevel};
use strategy_core::event::{
    log_event::NodeStateLogEvent,
    strategy_event::{StrategyPerformanceUpdateEvent, StrategyRunningLogEvent, StrategyStateLogEvent},
};
use strategy_stats::event::StrategyStatsUpdatedEvent;
use strum::Display;

use super::node_event::{
    futures_order_node_event::{
        FuturesOrderCanceledEvent, FuturesOrderCreatedEvent, FuturesOrderFilledEvent, StopLossOrderCanceledEvent,
        StopLossOrderCreatedEvent, StopLossOrderFilledEvent, TakeProfitOrderCanceledEvent, TakeProfitOrderCreatedEvent,
        TakeProfitOrderFilledEvent, TransactionCreatedEvent,
    },
    indicator_node_event::IndicatorUpdateEvent,
    kline_node_event::KlineUpdateEvent,
    position_node_event::{PositionClosedEvent, PositionCreatedEvent, PositionUpdatedEvent},
    variable_node_event::{CustomVarUpdateEvent, SysVarUpdateEvent},
};

#[derive(Debug, Clone, Serialize, Display, From)]
#[serde(tag = "event")]
pub enum BacktestStrategyEvent {
    #[strum(serialize = "play-finished-event")]
    #[serde(rename = "play-finished-event")]
    PlayFinished(PlayFinishedEvent), // 回测播放完毕事件

    #[strum(serialize = "kline-update-event")]
    #[serde(rename = "kline-update-event")]
    KlineUpdate(KlineUpdateEvent), // 回测K线更新事件

    #[strum(serialize = "indicator-update-event")]
    #[serde(rename = "indicator-update-event")]
    IndicatorUpdate(IndicatorUpdateEvent), // 回测指标更新事件

    #[strum(serialize = "sys-variable-update-event")]
    #[serde(rename = "sys-variable-update-event")]
    SysVariableUpdate(SysVarUpdateEvent), // 系统变量更新事件

    #[strum(serialize = "custom-variable-update-event")]
    #[serde(rename = "custom-variable-update-event")]
    CustomVariableUpdate(CustomVarUpdateEvent), // 自定义变量更新事件

    #[strum(serialize = "futures-order-filled-event")]
    #[serde(rename = "futures-order-filled-event")]
    FuturesOrderFilled(FuturesOrderFilledEvent), // 期货订单成交事件

    #[strum(serialize = "futures-order-created-event")]
    #[serde(rename = "futures-order-created-event")]
    FuturesOrderCreated(FuturesOrderCreatedEvent), // 期货订单创建事件

    #[strum(serialize = "futures-order-canceled-event")]
    #[serde(rename = "futures-order-canceled-event")]
    FuturesOrderCanceled(FuturesOrderCanceledEvent), // 期货订单取消事件

    #[strum(serialize = "take-profit-order-created-event")]
    #[serde(rename = "take-profit-order-created-event")]
    TakeProfitOrderCreated(TakeProfitOrderCreatedEvent), // 止盈订单创建事件

    #[strum(serialize = "take-profit-order-filled-event")]
    #[serde(rename = "take-profit-order-filled-event")]
    TakeProfitOrderFilled(TakeProfitOrderFilledEvent), // 止盈订单成交事件

    #[strum(serialize = "take-profit-order-canceled-event")]
    #[serde(rename = "take-profit-order-canceled-event")]
    TakeProfitOrderCanceled(TakeProfitOrderCanceledEvent), // 止盈订单取消事件

    #[strum(serialize = "stop-loss-order-created-event")]
    #[serde(rename = "stop-loss-order-created-event")]
    StopLossOrderCreated(StopLossOrderCreatedEvent), // 止损订单创建事件

    #[strum(serialize = "stop-loss-order-filled-event")]
    #[serde(rename = "stop-loss-order-filled-event")]
    StopLossOrderFilled(StopLossOrderFilledEvent), // 止损订单成交事件

    #[strum(serialize = "stop-loss-order-canceled-event")]
    #[serde(rename = "stop-loss-order-canceled-event")]
    StopLossOrderCanceled(StopLossOrderCanceledEvent), // 止损订单取消事件

    #[strum(serialize = "position-created-event")]
    #[serde(rename = "position-created-event")]
    PositionCreated(PositionCreatedEvent), // 仓位创建事件

    #[strum(serialize = "position-updated-event")]
    #[serde(rename = "position-updated-event")]
    PositionUpdated(PositionUpdatedEvent), // 仓位更新事件

    #[strum(serialize = "position-closed-event")]
    #[serde(rename = "position-closed-event")]
    PositionClosed(PositionClosedEvent), // 仓位关闭事件

    #[strum(serialize = "strategy-stats-updated-event")]
    #[serde(rename = "strategy-stats-updated-event")]
    StrategyStatsUpdated(StrategyStatsUpdatedEvent), // 策略统计更新事件

    #[strum(serialize = "transaction-created-event")]
    #[serde(rename = "transaction-created-event")]
    TransactionCreated(TransactionCreatedEvent), // 交易明细创建事件

    #[strum(serialize = "node-state-log-update-event")]
    #[serde(rename = "node-state-log-update-event")]
    NodeStateLog(NodeStateLogEvent), // 节点状态日志事件

    #[strum(serialize = "strategy-state-log-update-event")]
    #[serde(rename = "strategy-state-log-update-event")]
    StrategyStateLog(StrategyStateLogEvent), // 策略状态日志事件

    #[strum(serialize = "strategy-running-log-update-event")]
    #[serde(rename = "strategy-running-log-update-event")]
    RunningLog(StrategyRunningLogEvent), // 运行日志事件

    #[strum(serialize = "strategy-performance-update-event")]
    #[serde(rename = "strategy-performance-update-event")]
    StrategyPerformanceUpdate(StrategyPerformanceUpdateEvent), // 策略性能更新事件
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct StrategyStateLogEvent {
//     pub strategy_id: i32,

//     pub strategy_name: String,

//     pub strategy_state: Option<String>,

//     pub strategy_state_action: Option<String>,

//     pub log_level: LogLevel,

//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub error_code: Option<String>,

//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub error_code_chain: Option<Vec<String>>,

//     pub message: String,

//     pub datetime: DateTimeUtc,
// }

// impl StrategyStateLogEvent {
//     pub fn new(
//         strategy_id: i32,
//         strategy_name: String,
//         strategy_state: Option<String>,
//         strategy_state_action: Option<String>,
//         log_level: LogLevel,
//         error_code: Option<String>,
//         error_code_chain: Option<Vec<String>>,
//         message: String,
//     ) -> Self {
//         Self {
//             strategy_id,
//             strategy_name,
//             strategy_state,
//             strategy_state_action,
//             log_level,
//             error_code,
//             error_code_chain,
//             message,
//             datetime: Utc::now(),
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayFinishedEvent {
    pub strategy_id: i32,
    pub strategy_name: String,
    pub play_index: i32,
    pub datetime: DateTimeUtc,
}

impl PlayFinishedEvent {
    pub fn new(strategy_id: i32, strategy_name: String, play_index: i32) -> Self {
        Self {
            strategy_id,
            strategy_name,
            play_index,
            datetime: Utc::now(),
        }
    }
}

// // 策略性能更新时间
// #[derive(Debug, Clone, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct StrategyPerformanceUpdateEvent {
//     pub strategy_id: StrategyId,
//     pub report: StrategyPerformanceReport,
// }

// impl StrategyPerformanceUpdateEvent {
//     pub fn new(strategy_id: StrategyId, report: StrategyPerformanceReport) -> Self {
//         Self {
//             strategy_id,
//             report,
//         }
//     }
// }

// pub mod log_event {

//     use derive_more::From;
//     use serde::{Deserialize, Serialize};
//     use strum::Display;
//     use utoipa::ToSchema;
//     use chrono::{DateTime, Utc};
//     use star_river_core::error::error_trait::{ErrorLanguage, StarRiverErrorTrait};

//     #[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
//     #[serde(rename_all = "lowercase")]
//     pub enum LogLevel {
//         Trace,
//         Debug,
//         Info,
//         Warn,
//         Error,
//     }

//     #[derive(Debug, Clone, Serialize, Deserialize, From)]
//     #[serde(rename_all = "camelCase")]
//     pub struct NodeStateLogEvent {
//         pub strategy_id: i32,

//         pub node_id: String,

//         pub node_name: String,

//         pub node_state: String,

//         pub node_state_action: String,

//         pub log_level: LogLevel,

//         #[serde(skip_serializing_if = "Option::is_none")]
//         pub error_code: Option<String>,

//         #[serde(skip_serializing_if = "Option::is_none")]
//         pub error_code_chain: Option<Vec<String>>,

//         pub message: String,
//         pub datetime: DateTime<Utc>,
//     }

//     impl NodeStateLogEvent {
//         pub fn success(
//             strategy_id: i32,
//             node_id: String,
//             node_name: String,
//             node_state: String,
//             node_state_action: String,
//             message: String,
//         ) -> Self {
//             Self {
//                 strategy_id,
//                 node_id,
//                 node_name,
//                 node_state,
//                 node_state_action,
//                 log_level: LogLevel::Info,
//                 message,
//                 error_code: None,
//                 error_code_chain: None,
//                 datetime: Utc::now(),
//             }
//         }

//         pub fn error(
//             strategy_id: i32,
//             node_id: String,
//             node_name: String,
//             node_state: String,
//             node_state_action: String,
//             error: &impl StarRiverErrorTrait,
//         ) -> Self {
//             Self {
//                 strategy_id,
//                 node_id,
//                 node_name,
//                 node_state,
//                 node_state_action,
//                 log_level: LogLevel::Error,
//                 message: error.error_message(ErrorLanguage::Chinese),
//                 error_code: Some(error.error_code().to_string()),
//                 error_code_chain: Some(error.error_code_chain()),
//                 datetime: Utc::now(),
//             }
//         }
//     }

//     #[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
//     pub enum StrategyRunningLogSource {
//         #[strum(serialize = "node")]
//         #[serde(rename = "Node")]
//         Node,
//         #[strum(serialize = "virtual_trading_system")]
//         #[serde(rename = "VirtualTradingSystem")]
//         VirtualTradingSystem,
//     }

//     #[derive(Debug, Clone, Serialize, Deserialize, Display, ToSchema)]
//     pub enum StrategyRunningLogType {
//         #[strum(serialize = "condition_match")]
//         #[serde(rename = "ConditionMatch")]
//         ConditionMatch,
//         #[strum(serialize = "order_created")]
//         #[serde(rename = "OrderCreated")]
//         OrderCreated,
//         #[strum(serialize = "order_filled")]
//         #[serde(rename = "OrderFilled")]
//         OrderFilled,
//         #[strum(serialize = "order_canceled")]
//         #[serde(rename = "OrderCanceled")]
//         OrderCanceled,
//         #[strum(serialize = "processing_order")]
//         #[serde(rename = "ProcessingOrder")]
//         ProcessingOrder,
//     }

//     // 策略运行日志
//     #[derive(Debug, Clone, Serialize, Deserialize, ToSchema, From)]
//     pub struct StrategyRunningLogEvent {
//         #[serde(rename = "strategyId")]
//         pub strategy_id: i32,

//         #[serde(rename = "nodeId")]
//         pub node_id: String,

//         #[serde(rename = "nodeName")]
//         pub node_name: String,

//         #[serde(rename = "source")]
//         pub source: StrategyRunningLogSource,

//         #[serde(rename = "logLevel")]
//         pub log_level: LogLevel,

//         #[serde(rename = "logType")]
//         pub log_type: StrategyRunningLogType,

//         #[serde(rename = "message")]
//         pub message: String,

//         #[serde(rename = "detail")]
//         pub detail: serde_json::Value,

//         #[serde(rename = "errorCode")]
//         pub error_code: Option<String>,

//         #[serde(rename = "errorCodeChain")]
//         pub error_code_chain: Option<Vec<String>>,

//         #[serde(rename = "datetime")]
//         #[schema(value_type = String, example = "2024-01-01T12:00:00Z")]
//         pub datetime: DateTime<Utc>,
//     }

//     impl StrategyRunningLogEvent {
//         pub fn success(
//             strategy_id: i32,
//             node_id: String,
//             node_name: String,
//             source: StrategyRunningLogSource,
//             log_type: StrategyRunningLogType,
//             message: String,
//             detail: serde_json::Value,
//             datetime: DateTime<Utc>,
//         ) -> Self {
//             Self {
//                 strategy_id,
//                 node_id,
//                 node_name,
//                 source,
//                 log_level: LogLevel::Info,
//                 log_type,
//                 message,
//                 detail,
//                 error_code: None,
//                 error_code_chain: None,
//                 datetime,
//             }
//         }

//         pub fn warn(
//             strategy_id: i32,
//             node_id: String,
//             node_name: String,
//             source: StrategyRunningLogSource,
//             log_type: StrategyRunningLogType,
//             message: String,
//             detail: serde_json::Value,
//             datetime: DateTime<Utc>,
//         ) -> Self {
//             Self {
//                 strategy_id,
//                 node_id,
//                 node_name,
//                 source,
//                 log_level: LogLevel::Warn,
//                 log_type,
//                 message,
//                 detail,
//                 error_code: None,
//                 error_code_chain: None,
//                 datetime,
//             }
//         }
//     }

// }
