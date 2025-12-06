use chrono::Utc;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::system::DateTimeUtc;
// use log_event::{NodeStateLogEvent, StrategyRunningLogEvent, LogLevel};
use strategy_core::event::{
    log_event::NodeRunStateLogEvent,
    node_common_event::NodeRunningLogEvent,
    strategy_event::{StrategyPerformanceUpdateEvent, StrategyRunningLogEvent, StrategyStateLogEvent},
};
use strategy_stats::event::StrategyStatsUpdatedEvent;
use strum::Display;
use virtual_trading::types::{VirtualOrder, VirtualPosition, VirtualTransaction};

use super::node_event::{
    indicator_node_event::IndicatorUpdateEvent,
    kline_node_event::KlineUpdateEvent,
    variable_node_event::{CustomVarUpdateEvent, SysVarUpdateEvent},
};

#[derive(Debug, Clone, Serialize, Display, From)]
#[serde(tag = "event")]
pub enum BacktestStrategyEvent {
    #[strum(serialize = "play-finished-event")]
    #[serde(rename = "play-finished-event")]
    PlayFinished(PlayFinishedEvent), // Backtest playback finished event

    #[strum(serialize = "kline-update-event")]
    #[serde(rename = "kline-update-event")]
    KlineUpdate(KlineUpdateEvent), // Backtest kline update event

    #[strum(serialize = "indicator-update-event")]
    #[serde(rename = "indicator-update-event")]
    IndicatorUpdate(IndicatorUpdateEvent), // Backtest indicator update event

    #[strum(serialize = "sys-variable-update-event")]
    #[serde(rename = "sys-variable-update-event")]
    SysVariableUpdate(SysVarUpdateEvent), // System variable update event

    #[strum(serialize = "custom-variable-update-event")]
    #[serde(rename = "custom-variable-update-event")]
    CustomVariableUpdate(CustomVarUpdateEvent), // Custom variable update event

    #[strum(serialize = "futures-order-filled-event")]
    #[serde(rename = "futures-order-filled-event")]
    FuturesOrderFilled {
        #[serde(rename = "futuresOrder")]
        futures_order: VirtualOrder,
    }, // Futures order filled event

    #[strum(serialize = "futures-order-created-event")]
    #[serde(rename = "futures-order-created-event")]
    #[from(ignore)]
    FuturesOrderCreated {
        #[serde(rename = "futuresOrder")]
        futures_order: VirtualOrder,
    }, // Futures order created event

    #[strum(serialize = "futures-order-canceled-event")]
    #[serde(rename = "futures-order-canceled-event")]
    #[from(ignore)]
    FuturesOrderCanceled {
        #[serde(rename = "futuresOrder")]
        futures_order: VirtualOrder,
    }, // Futures order canceled event

    #[strum(serialize = "take-profit-order-created-event")]
    #[serde(rename = "take-profit-order-created-event")]
    #[from(ignore)]
    TakeProfitOrderCreated {
        #[serde(rename = "takeProfitOrder")]
        take_profit_order: VirtualOrder,
    }, // Take profit order created event

    #[strum(serialize = "take-profit-order-filled-event")]
    #[serde(rename = "take-profit-order-filled-event")]
    #[from(ignore)]
    TakeProfitOrderFilled {
        #[serde(rename = "takeProfitOrder")]
        take_profit_order: VirtualOrder,
    }, // Take profit order filled event

    #[strum(serialize = "take-profit-order-canceled-event")]
    #[serde(rename = "take-profit-order-canceled-event")]
    #[from(ignore)]
    TakeProfitOrderCanceled {
        #[serde(rename = "takeProfitOrder")]
        take_profit_order: VirtualOrder,
    }, // Take profit order canceled event

    #[strum(serialize = "stop-loss-order-created-event")]
    #[serde(rename = "stop-loss-order-created-event")]
    #[from(ignore)]
    StopLossOrderCreated {
        #[serde(rename = "stopLossOrder")]
        stop_loss_order: VirtualOrder,
    }, // Stop loss order created event

    #[strum(serialize = "stop-loss-order-filled-event")]
    #[serde(rename = "stop-loss-order-filled-event")]
    #[from(ignore)]
    StopLossOrderFilled {
        #[serde(rename = "stopLossOrder")]
        stop_loss_order: VirtualOrder,
    }, // Stop loss order filled event

    #[strum(serialize = "stop-loss-order-canceled-event")]
    #[serde(rename = "stop-loss-order-canceled-event")]
    #[from(ignore)]
    StopLossOrderCanceled {
        #[serde(rename = "stopLossOrder")]
        stop_loss_order: VirtualOrder,
    }, // Stop loss order canceled event

    #[strum(serialize = "position-created-event")]
    #[serde(rename = "position-created-event")]
    PositionCreated {
        #[serde(rename = "virtualPosition")]
        virtual_position: VirtualPosition,
    }, // Position created event

    #[strum(serialize = "position-updated-event")]
    #[serde(rename = "position-updated-event")]
    #[from(ignore)]
    PositionUpdated {
        #[serde(rename = "virtualPosition")]
        virtual_position: VirtualPosition,
    }, // Position updated event

    #[strum(serialize = "position-closed-event")]
    #[serde(rename = "position-closed-event")]
    #[from(ignore)]
    PositionClosed {
        #[serde(rename = "virtualPosition")]
        virtual_position: VirtualPosition,
    }, // Position closed event

    #[strum(serialize = "transaction-created-event")]
    #[serde(rename = "transaction-created-event")]
    TransactionCreated {
        #[serde(rename = "transaction")]
        transaction: VirtualTransaction,
    }, // Transaction created event

    #[strum(serialize = "strategy-stats-updated-event")]
    #[serde(rename = "strategy-stats-updated-event")]
    StrategyStatsUpdated(StrategyStatsUpdatedEvent), // Strategy stats updated event

    #[strum(serialize = "node-state-log-update-event")]
    #[serde(rename = "node-state-log-update-event")]
    NodeStateLog(NodeRunStateLogEvent), // Node state log event

    #[strum(serialize = "strategy-state-log-update-event")]
    #[serde(rename = "strategy-state-log-update-event")]
    StrategyStateLog(StrategyStateLogEvent), // Strategy state log event

    #[strum(serialize = "node-running-log-update-event")]
    #[serde(rename = "node-running-log-update-event")]
    NodeRunningLog(NodeRunningLogEvent), // Node running log event

    #[strum(serialize = "strategy-running-log-update-event")]
    #[serde(rename = "strategy-running-log-update-event")]
    StrategyRunningLog(StrategyRunningLogEvent), // Strategy running log event

    #[strum(serialize = "strategy-performance-update-event")]
    #[serde(rename = "strategy-performance-update-event")]
    StrategyPerformanceUpdate(StrategyPerformanceUpdateEvent), // Strategy performance update event
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
