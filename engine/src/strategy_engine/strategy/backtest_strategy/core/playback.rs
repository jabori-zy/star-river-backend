use super::{BacktestStrategy, BacktestStrategyError};

impl BacktestStrategy {
    pub async fn play(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        tracing::info!("[{}({})] start play kline", strategy_name, strategy_id);
        let mut context_guard = self.context.write().await;
        context_guard.play().await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), BacktestStrategyError> {
        let mut context_guard = self.context.write().await;
        context_guard.pause().await?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        tracing::info!("[{}({})] reset play", strategy_name, strategy_id);
        let mut context_guard = self.context.write().await;
        context_guard.reset().await?;
        // 重置虚拟交易系统
        context_guard.virtual_trading_system_reset().await;
        // 重置策略统计
        context_guard.strategy_stats_reset().await;
        context_guard.send_reset_node_event().await;

        Ok(())
    }

    pub async fn play_one_kline(&mut self) -> Result<i32, BacktestStrategyError> {
        let mut context_guard = self.context.write().await;
        let play_index = context_guard.play_one_kline().await?;
        Ok(play_index)
    }
}
