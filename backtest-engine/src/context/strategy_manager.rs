use database::query::strategy_config_query::StrategyConfigQuery;
use star_river_core::error::StarRiverErrorTrait;
use strategy_core::strategy::{StrategyConfig, TradeMode};

use super::BacktestEngineContext;
use crate::{
    engine_error::{BacktestEngineError, UnsupportedTradeModeSnafu},
    strategy::strategy_state_machine::BacktestStrategyRunState,
};

impl BacktestEngineContext {
    pub async fn get_strategy_info_by_id(&self, id: i32) -> Result<StrategyConfig, BacktestEngineError> {
        let strategy = StrategyConfigQuery::get_strategy_by_id(&self.database, id).await?;
        Ok(strategy)
    }

    pub async fn remove_strategy_instance(&mut self, trade_mode: TradeMode, strategy_id: i32) -> Result<(), BacktestEngineError> {
        match trade_mode {
            TradeMode::Backtest => {
                self.strategy_list.lock().await.remove(&strategy_id);
                tracing::info!("backtest strategy [{}] instance is removed", strategy_id);
            }
            _ => {
                let error = UnsupportedTradeModeSnafu {
                    trade_mode: trade_mode.to_string(),
                }
                .build();
                error.report_log();
                return Err(error);
            }
        }
        Ok(())
    }

    pub async fn get_strategy_run_state(&self, strategy_id: i32) -> Result<String, BacktestEngineError> {
        // 检查是否正在初始化或有策略实例
        let is_initializing = self.initializing_strategies.lock().await.contains(&strategy_id);
        let has_instance = self.strategy_list.lock().await.contains_key(&strategy_id);
        let strategy_status = StrategyConfigQuery::get_strategy_run_state(&self.database, strategy_id).await?;

        if is_initializing || has_instance {
            // 正在初始化或有实例，返回数据库中的状态
            Ok(strategy_status)
        }
        // 无实例且未初始化, 但是状态为running，则将状态设为stopped
        else if !is_initializing && !has_instance && strategy_status != BacktestStrategyRunState::Error.to_string() {
            // 无实例且未初始化，将状态设为stopped并返回
            // StrategyConfigMutation::update_strategy_status(&self.database, strategy_id, BacktestStrategyRunState::Stopped.to_string()).await?;
            Ok(BacktestStrategyRunState::Stopped.to_string())
        } else {
            Ok(strategy_status)
        }
    }
}
