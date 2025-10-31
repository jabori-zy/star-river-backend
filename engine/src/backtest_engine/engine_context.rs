pub mod strategy_manager;
pub mod context_impl;

use crate::backtest_engine::strategy::BacktestStrategy;
use crate::{EngineContext, EngineName};
use database::query::strategy_config_query::StrategyConfigQuery;
use event_center::communication::engine::EngineCommand;
use event_center::event::Event;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use snafu::{Report, ResultExt};
use star_river_core::{
    custom_type::StrategyId,
    error::engine_error::strategy_engine_error::*,
    strategy::{StrategyConfig, TradeMode},
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct StrategyEngineContext {
    pub engine_name: EngineName,
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub strategy_list: Arc<Mutex<HashMap<StrategyId, BacktestStrategy>>>, // 回测策略列表
    pub initializing_strategies: Arc<Mutex<HashSet<StrategyId>>>,
}

impl Clone for StrategyEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            strategy_list: self.strategy_list.clone(),
            database: self.database.clone(),
            heartbeat: self.heartbeat.clone(),
            initializing_strategies: self.initializing_strategies.clone(),
        }
    }
}

impl StrategyEngineContext {
    pub async fn get_strategy_instance(&self, strategy_id: StrategyId) -> Result<BacktestStrategy, StrategyEngineError> {
        let backtest_strategy_list = self.strategy_list.lock().await;
        if let Some(strategy) = backtest_strategy_list.get(&strategy_id) {
            Ok(strategy.clone())
        } else {
            let error = StrategyInstanceNotFoundSnafu { strategy_id }.build();
            let report = Report::from_error(&error);
            tracing::error!("{}", report);
            Err(error)
        }
    }

    // 注意：由于 backtest_strategy_list 是 Arc<Mutex<HashMap<...>>>，
    // 无法直接返回可变引用。如需修改策略，请考虑使用其他方法。
    // pub async fn get_backtest_strategy_instance_mut(&mut self, strategy_id: StrategyId) -> Result<&mut BacktestStrategy, String> {
    //     // 此方法无法实现，因为无法返回指向 Mutex 保护数据的可变引用
    // }

    pub async fn get_strategy_info_by_id(&self, id: i32) -> Result<StrategyConfig, StrategyEngineError> {
        let strategy = StrategyConfigQuery::get_strategy_by_id(&self.database, id)
            .await
            .context(StrategyConfigNotFoundSnafu { strategy_id: id })?;
        Ok(strategy)
    }

    pub async fn remove_strategy_instance(&mut self, trade_mode: TradeMode, strategy_id: i32) -> Result<(), StrategyEngineError> {
        match trade_mode {
            TradeMode::Backtest => {
                self.strategy_list.lock().await.remove(&strategy_id);
                tracing::info!("backtest strategy [{}] instance is removed", strategy_id);
            }
            _ => {
                tracing::error!("backtest strategy engine not support trade mode: {}", trade_mode);
                return Err(UnsupportedTradeModeSnafu {
                    trade_mode: trade_mode.to_string(),
                }
                .build());
            }
        }
        Ok(())
    }
}
