use super::{BacktestStrategyEngine, TradeMode, StrategyEngineContext};
use event_center::event::strategy_event::StrategyRunningLogEvent;
use star_river_core::{
    custom_type::{PlayIndex, StrategyId, NodeId},
    error::engine_error::strategy_engine_error::*,
    key::Key,
    order::virtual_order::VirtualOrder,
    position::virtual_position::VirtualPosition,
    strategy::{
        strategy_benchmark::StrategyPerformanceReport, StrategyVariable
    },
    strategy_stats::StatsSnapshot,
    system::DateTimeUtc,
    transaction::virtual_transaction::VirtualTransaction,
};
use std::collections::HashMap;

/// 数据查询方法
/// 所有查询方法都使用私有的 get_strategy_context() 辅助方法直接访问策略上下文
/// 避免了层层转发调用，简化了调用链
impl BacktestStrategyEngine {

    // 获取策略缓存键
    pub async fn get_strategy_cache_keys(&mut self, strategy_id: StrategyId) -> Result<HashMap<Key, NodeId>, StrategyEngineError> {
        let context = self.context.read().await;
        let strategy_context = context.as_any().downcast_ref::<StrategyEngineContext>().unwrap();
        let strategy_info = strategy_context.get_strategy_info_by_id(strategy_id).await?;
        match strategy_info.trade_mode {
            TradeMode::Backtest => Ok(strategy_context.get_backtest_strategy_keys(strategy_id).await),
            _ => Err(UnsupportedStrategyTypeSnafu {
                strategy_type: strategy_info.trade_mode.to_string(),
            }
            .build()),
        }
    }
    /// 获取策略性能报告
    pub async fn get_strategy_performance_report(
        &self,
        strategy_id: StrategyId,
    ) -> Result<StrategyPerformanceReport, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_strategy_performance_report().await)
    }

    /// 获取播放索引
    pub async fn get_play_index(&self, strategy_id: StrategyId) -> Result<i32, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_play_index().await)
    }

    /// 获取虚拟订单列表
    pub async fn get_virtual_orders(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<VirtualOrder>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_virtual_orders().await)
    }

    /// 获取当前持仓
    pub async fn get_current_virtual_positions(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<VirtualPosition>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_current_positions().await)
    }

    /// 获取历史持仓
    pub async fn get_history_virtual_positions(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<VirtualPosition>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_history_positions().await)
    }

    /// 获取统计快照历史
    pub async fn get_stats_history(
        &self,
        strategy_id: StrategyId,
        play_index: PlayIndex,
    ) -> Result<Vec<StatsSnapshot>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_stats_history(play_index).await)
    }

    /// 获取交易明细
    pub async fn get_virtual_transactions(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<VirtualTransaction>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_transactions().await)
    }

    /// 获取运行日志
    pub async fn get_running_log(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<StrategyRunningLogEvent>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_running_log().await)
    }

    /// 根据播放索引获取策略数据
    pub async fn get_strategy_data(
        &self,
        strategy_id: StrategyId,
        play_index: PlayIndex,
        key: Key,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_strategy_data(play_index, key, limit).await?)
    }

    /// 根据日期时间获取策略数据
    pub async fn get_strategy_data_by_datetime(
        &self,
        strategy_id: StrategyId,
        key: Key,
        datetime: DateTimeUtc,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard
            .get_strategy_data_by_datetime(key, datetime, limit)
            .await?)
    }

    /// 获取策略变量
    pub async fn get_strategy_variable(
        &self,
        strategy_id: StrategyId,
    ) -> Result<Vec<StrategyVariable>, StrategyEngineError> {
        let ctx = self.get_strategy_context(strategy_id).await?;
        let ctx_guard = ctx.read().await;
        Ok(ctx_guard.get_strategy_variable().await)
    }
}
