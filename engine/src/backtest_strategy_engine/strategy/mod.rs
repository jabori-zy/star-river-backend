pub(super) mod strategy_context;
pub(super) mod workflow;
pub(super) mod strategy_log_message;
pub(super) mod strategy_core;
pub(super) mod strategy_state_machine;

use super::node::*;
use strategy_context::BacktestStrategyContext;
use workflow::BacktestStrategyFunction;
use event_center::event::strategy_event::StrategyRunningLogEvent;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::*;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::key::key::KlineKey;
use star_river_core::key::{Key, KeyTrait};
use star_river_core::market::KlineInterval;
use star_river_core::order::virtual_order::VirtualOrder;
use star_river_core::position::virtual_position::VirtualPosition;
use star_river_core::strategy::StrategyConfig;
use strategy_state_machine::*;
use strategy_state_machine::{BacktestStrategyStateAction, BacktestStrategyStateMachine};
use virtual_trading::VirtualTradingSystem;

use strategy_log_message::StrategyStateLogMsg;
use event_center::EventCenterSingleton;
use event_center::event::strategy_event::LogLevel;
use event_center::event::strategy_event::backtest_strategy_event::{BacktestStrategyEvent, StrategyStateLogEvent};
use star_river_core::strategy_stats::StatsSnapshot;
use star_river_core::transaction::virtual_transaction::VirtualTransaction;
use std::sync::Arc;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<BacktestStrategyContext>>,
}

impl BacktestStrategy {
    pub async fn new(strategy_config: StrategyConfig, database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        let context = BacktestStrategyContext::new(strategy_config, database, heartbeat);
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}

impl BacktestStrategy {
    pub fn get_context(&self) -> Arc<RwLock<BacktestStrategyContext>> {
        self.context.clone()
    }

    pub async fn get_strategy_id(&self) -> i32 {
        self.context.read().await.strategy_id
    }

    pub async fn get_strategy_name(&self) -> String {
        self.context.read().await.strategy_name.clone()
    }

    pub async fn get_state_machine(&self) -> BacktestStrategyStateMachine {
        self.context.read().await.state_machine.clone()
    }
}
