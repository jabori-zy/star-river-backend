use chrono::Utc;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::system::DateTimeUtc;
// use log_event::{NodeStateLogEvent, StrategyRunningLogEvent, LogLevel};
use strategy_core::event::{
    log_event::NodeStateLogEvent,
    node_common_event::NodeRunningLogEvent,
    strategy_event::{StrategyPerformanceUpdateEvent, StrategyRunningLogEvent, StrategyStateLogEvent},
};
use strategy_stats::event::StrategyStatsUpdatedEvent;
use strum::Display;
use virtual_trading::types::{VirtualOrder, VirtualPosition, VirtualTransaction};

use super::node_event::{
    futures_order_node_event::{
        FuturesOrderCanceledEvent, FuturesOrderCreatedEvent, FuturesOrderFilledEvent, StopLossOrderCanceledEvent,
        StopLossOrderCreatedEvent, StopLossOrderFilledEvent, TakeProfitOrderCanceledEvent, TakeProfitOrderCreatedEvent,
        TakeProfitOrderFilledEvent, TransactionCreatedEvent,
    },
    indicator_node_event::IndicatorUpdateEvent,
    kline_node_event::KlineUpdateEvent,
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
    FuturesOrderFilled {
        #[serde(rename = "futuresOrder")]
        futures_order: VirtualOrder,
    }, // 期货订单成交事件

    #[strum(serialize = "futures-order-created-event")]
    #[serde(rename = "futures-order-created-event")]
    #[from(ignore)]
    FuturesOrderCreated {
        #[serde(rename = "futuresOrder")]
        futures_order: VirtualOrder,
    }, // 期货订单创建事件

    #[strum(serialize = "futures-order-canceled-event")]
    #[serde(rename = "futures-order-canceled-event")]
    #[from(ignore)]
    FuturesOrderCanceled {
        #[serde(rename = "futuresOrder")]
        futures_order: VirtualOrder,
    }, // 期货订单取消事件

    #[strum(serialize = "take-profit-order-created-event")]
    #[serde(rename = "take-profit-order-created-event")]
    #[from(ignore)]
    TakeProfitOrderCreated {
        #[serde(rename = "takeProfitOrder")]
        take_profit_order: VirtualOrder,
    }, // 止盈订单创建事件

    #[strum(serialize = "take-profit-order-filled-event")]
    #[serde(rename = "take-profit-order-filled-event")]
    #[from(ignore)]
    TakeProfitOrderFilled {
        #[serde(rename = "takeProfitOrder")]
        take_profit_order: VirtualOrder,
    }, // 止盈订单成交事件

    #[strum(serialize = "take-profit-order-canceled-event")]
    #[serde(rename = "take-profit-order-canceled-event")]
    #[from(ignore)]
    TakeProfitOrderCanceled {
        #[serde(rename = "takeProfitOrder")]
        take_profit_order: VirtualOrder,
    }, // 止盈订单取消事件

    #[strum(serialize = "stop-loss-order-created-event")]
    #[serde(rename = "stop-loss-order-created-event")]
    #[from(ignore)]
    StopLossOrderCreated {
        #[serde(rename = "stopLossOrder")]
        stop_loss_order: VirtualOrder,
    }, // 止损订单创建事件

    #[strum(serialize = "stop-loss-order-filled-event")]
    #[serde(rename = "stop-loss-order-filled-event")]
    #[from(ignore)]
    StopLossOrderFilled {
        #[serde(rename = "stopLossOrder")]
        stop_loss_order: VirtualOrder,
    }, // 止损订单成交事件

    #[strum(serialize = "stop-loss-order-canceled-event")]
    #[serde(rename = "stop-loss-order-canceled-event")]
    #[from(ignore)]
    StopLossOrderCanceled {
        #[serde(rename = "stopLossOrder")]
        stop_loss_order: VirtualOrder,
    }, // 止损订单取消事件

    #[strum(serialize = "position-created-event")]
    #[serde(rename = "position-created-event")]
    PositionCreated {
        #[serde(rename = "virtualPosition")]
        virtual_position: VirtualPosition,
    }, // 仓位创建事件

    #[strum(serialize = "position-updated-event")]
    #[serde(rename = "position-updated-event")]
    #[from(ignore)]
    PositionUpdated {
        #[serde(rename = "virtualPosition")]
        virtual_position: VirtualPosition,
    }, // 仓位更新事件

    #[strum(serialize = "position-closed-event")]
    #[serde(rename = "position-closed-event")]
    #[from(ignore)]
    PositionClosed {
        #[serde(rename = "virtualPosition")]
        virtual_position: VirtualPosition,
    }, // 仓位关闭事件

    #[strum(serialize = "transaction-created-event")]
    #[serde(rename = "transaction-created-event")]
    TransactionCreated {
        #[serde(rename = "transaction")]
        transaction: VirtualTransaction,
    }, // 交易明细创建事件

    #[strum(serialize = "strategy-stats-updated-event")]
    #[serde(rename = "strategy-stats-updated-event")]
    StrategyStatsUpdated(StrategyStatsUpdatedEvent), // 策略统计更新事件

    #[strum(serialize = "node-state-log-update-event")]
    #[serde(rename = "node-state-log-update-event")]
    NodeStateLog(NodeStateLogEvent), // 节点状态日志事件

    #[strum(serialize = "strategy-state-log-update-event")]
    #[serde(rename = "strategy-state-log-update-event")]
    StrategyStateLog(StrategyStateLogEvent), // 策略状态日志事件

    #[strum(serialize = "node-running-log-update-event")]
    #[serde(rename = "node-running-log-update-event")]
    NodeRunningLog(NodeRunningLogEvent), // 运行日志事件

    #[strum(serialize = "strategy-running-log-update-event")]
    #[serde(rename = "strategy-running-log-update-event")]
    StrategyRunningLog(StrategyRunningLogEvent), // 运行日志事件

    #[strum(serialize = "strategy-performance-update-event")]
    #[serde(rename = "strategy-performance-update-event")]
    StrategyPerformanceUpdate(StrategyPerformanceUpdateEvent), // 策略性能更新事件
}

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
