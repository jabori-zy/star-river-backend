use super::BacktestStrategyEngine;
use crate::backtest_strategy_engine::engine_context::StrategyEngineContext;
use snafu::Report;
use star_river_core::custom_type::{PlayIndex, StrategyId};
use star_river_core::error::engine_error::strategy_engine_error::*;
use star_river_core::strategy::TradeMode;

/// Backtest strategy control methods
/// Using direct access pattern, consistent with strategy_data_query.rs
/// Directly call methods in BacktestStrategyContext to avoid multi-layer forwarding
impl BacktestStrategyEngine {
    /// Initialize strategy
    pub async fn init_strategy(&mut self, strategy_id: StrategyId) -> Result<(), StrategyEngineError> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await.unwrap();
        match strategy_info.trade_mode {
            TradeMode::Backtest => {
                if let Err(e) = strategy_context.init(strategy_id).await {
                    let report = Report::from_error(&e);
                    tracing::error!("{}", report);
                    return Err(e);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Stop strategy
    pub async fn stop_strategy(&mut self, strategy_id: StrategyId) -> Result<(), StrategyEngineError> {
        let mut context = self.context.write().await;
        let strategy_context = context.as_any_mut().downcast_mut::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await?;
        match strategy_info.trade_mode {
            TradeMode::Backtest => {
                strategy_context.stop(strategy_id).await?;
                Ok(())
            }
            _ => Err(UnsupportedStrategyTypeSnafu {
                strategy_type: strategy_info.trade_mode.to_string(),
            }
            .build()),
        }
    }

    /// Play strategy
    pub async fn play(&mut self, strategy_id: StrategyId) -> Result<(), StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let mut ctx_guard = ctx.write().await;
        Ok(ctx_guard.play().await?)
    }

    /// Pause strategy
    pub async fn pause(&mut self, strategy_id: StrategyId) -> Result<(), StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let mut ctx_guard = ctx.write().await;
        Ok(ctx_guard.pause().await?)
    }

    /// Reset strategy
    pub async fn reset(&mut self, strategy_id: StrategyId) -> Result<(), StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let mut ctx_guard = ctx.write().await;
        ctx_guard.reset().await?;
        // Reset virtual trading system
        ctx_guard.virtual_trading_system_reset().await;
        // Reset strategy stats
        ctx_guard.strategy_stats_reset().await;
        // Send reset node event
        ctx_guard.send_reset_node_event().await;
        Ok(())
    }

    /// Play one kline
    pub async fn play_one_kline(&mut self, strategy_id: StrategyId) -> Result<PlayIndex, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let mut ctx_guard = ctx.write().await;
        Ok(ctx_guard.play_one().await?)
    }

    /// Get strategy status
    pub async fn get_strategy_status(&mut self, strategy_id: StrategyId) -> Result<String, StrategyEngineError> {
        let context = self.context.read().await;
        let strategy_context = context.as_any().downcast_ref::<StrategyEngineContext>().unwrap();
        strategy_context.get_strategy_status(strategy_id).await
    }
}
