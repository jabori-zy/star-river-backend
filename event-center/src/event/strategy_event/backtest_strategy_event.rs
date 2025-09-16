use super::super::node_event::backtest_node_event::futures_order_node_event::{
    FuturesOrderCanceledEvent, FuturesOrderCreatedEvent, FuturesOrderFilledEvent,
    StopLossOrderCanceledEvent, StopLossOrderCreatedEvent, StopLossOrderFilledEvent,
    TakeProfitOrderCanceledEvent, TakeProfitOrderCreatedEvent, TakeProfitOrderFilledEvent,
    TransactionCreatedEvent,
};
use super::super::node_event::backtest_node_event::indicator_node_event::IndicatorUpdateEvent;
use super::super::node_event::backtest_node_event::kline_node_event::{KlineUpdateEvent};
use super::super::node_event::backtest_node_event::position_management_node_event::{
    PositionClosedEvent, PositionCreatedEvent, PositionUpdatedEvent,
};
use super::super::strategy_event::{LogLevel, NodeStateLogEvent, StrategyRunningLogEvent};
use crate::{event::Event, StrategyEvent};
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::PlayIndex;
use star_river_core::system::DateTimeUtc;
use star_river_core::strategy_stats::event::StrategyStatsUpdatedEvent;
use strum::Display;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
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
}

impl From<BacktestStrategyEvent> for Event {
    fn from(event: BacktestStrategyEvent) -> Self {
        StrategyEvent::BacktestStrategy(event).into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStateLogEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "strategyName")]
    pub strategy_name: String,

    #[serde(rename = "strategyState")]
    pub strategy_state: Option<String>,

    #[serde(rename = "strategyStateAction")]
    pub strategy_state_action: Option<String>,

    #[serde(rename = "logLevel")]
    pub log_level: LogLevel,

    #[serde(rename = "errorCode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    #[serde(rename = "errorCodeChain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code_chain: Option<Vec<String>>,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "datetime")]
    pub datetime: DateTimeUtc,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayFinishedEvent {
    #[serde(rename = "strategyId")]
    pub strategy_id: i32,

    #[serde(rename = "strategyName")]
    pub strategy_name: String,

    #[serde(rename = "playIndex")]
    pub play_index: PlayIndex,

    #[serde(rename = "datetime")]
    pub datetime: DateTimeUtc,
}

impl PlayFinishedEvent {
    pub fn new(strategy_id: i32, strategy_name: String, play_index: PlayIndex) -> Self {
        Self { strategy_id, strategy_name, play_index, datetime: Utc::now() }
    }
}

impl From<PlayFinishedEvent> for Event {
    fn from(event: PlayFinishedEvent) -> Self {
        BacktestStrategyEvent::PlayFinished(event).into()
    }
}
