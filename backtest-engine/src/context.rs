mod event_handler;
mod strategy_manager;
mod strategy_control;

// std
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

// third-party
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

// workspace crate
use engine_core::{
    context_trait::EngineContextTrait,
    EngineBaseContext,
};
use star_river_core::custom_type::StrategyId;

// current crate
use crate::{
    state_machine::BacktestEngineAction,
};
use crate::strategy_new::config::BacktestStrategyConfig;
use crate::strategy_new::BacktestStrategy;
use crate::strategy_new::strategy_context::BacktestStrategyContext;
use strategy_core::strategy::strategy_trait::StrategyContextAccessor;



#[derive(Debug, Clone)]
pub struct BacktestEngineContext {
    pub base_context: EngineBaseContext<BacktestEngineAction>,
    pub database: DatabaseConnection,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub strategy_list: Arc<Mutex<HashMap<StrategyId, BacktestStrategy>>>, // 回测策略列表
    pub initializing_strategies: Arc<Mutex<HashSet<StrategyId>>>,
}


impl BacktestEngineContext {
    pub fn new(
        base_context: EngineBaseContext<BacktestEngineAction>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {

        Self {
            base_context,
            database,
            heartbeat,
            strategy_list: Arc::new(Mutex::new(HashMap::new())),
            initializing_strategies: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn with_strategy<R, F>(&self, strategy_id: StrategyId, f: F) -> Result<R, crate::backtest_engine_error::BacktestEngineError>
    where
        F: for<'a> FnOnce(&'a BacktestStrategy) -> R + Send,
        R: Send,
    {
        use crate::backtest_engine_error::StrategyInstanceNotFoundSnafu;
        use snafu::OptionExt;

        let guard = self.strategy_list.lock().await;
        let strategy = guard.get(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy))
    }

    
    pub async fn with_strategy_mut<R, F>(&self, strategy_id: StrategyId, f: F) -> Result<R, crate::backtest_engine_error::BacktestEngineError>
    where
        F: for<'a> FnOnce(&'a mut BacktestStrategy) -> R + Send,
        R: Send,
    {
        use crate::backtest_engine_error::StrategyInstanceNotFoundSnafu;
        use snafu::OptionExt;

        let mut guard = self.strategy_list.lock().await;
        let strategy = guard.get_mut(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy))
    }

    
    pub async fn with_strategy_async<R>(&self, strategy_id: StrategyId, f: impl for<'a> FnOnce(&'a BacktestStrategy) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send) -> Result<R, crate::backtest_engine_error::BacktestEngineError>
    where
        R: Send,
    {
        use crate::backtest_engine_error::StrategyInstanceNotFoundSnafu;
        use snafu::OptionExt;

        let guard = self.strategy_list.lock().await;
        let strategy = guard.get(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy).await)
    }

    
    pub async fn with_strategy_mut_async<R>(&self, strategy_id: StrategyId, f: impl for<'a> FnOnce(&'a mut BacktestStrategy) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send) -> Result<R, crate::backtest_engine_error::BacktestEngineError>
    where
        R: Send,
    {
        use crate::backtest_engine_error::StrategyInstanceNotFoundSnafu;
        use snafu::OptionExt;

        let mut guard = self.strategy_list.lock().await;
        let strategy = guard.get_mut(&strategy_id).context(StrategyInstanceNotFoundSnafu { strategy_id })?;
        Ok(f(strategy).await)
    }

    /// 以读锁方式访问策略上下文（异步闭包）
    ///
    /// 这是一个辅助方法，直接访问策略的上下文，减少嵌套层级
    ///
    /// # 示例
    /// ```rust
    /// context.with_strategy_ctx_read_async(strategy_id, |ctx| {
    ///     Box::pin(async move {
    ///         ctx.get_virtual_orders().await
    ///     })
    /// }).await?;
    /// ```
    pub async fn with_strategy_ctx_read_async<R>(
        &self,
        strategy_id: StrategyId,
        f: impl for<'a> FnOnce(&'a BacktestStrategyContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send + 'static
    ) -> Result<R, crate::backtest_engine_error::BacktestEngineError>
    where
        R: Send,
    {
        self.with_strategy_async(strategy_id, |strategy| {
            Box::pin(async move {
                strategy.with_ctx_read_async(f).await
            })
        }).await
    }

    /// 以写锁方式访问策略上下文（异步闭包）
    ///
    /// 这是一个辅助方法，直接访问策略的上下文，减少嵌套层级
    ///
    /// # 示例
    /// ```rust
    /// context.with_strategy_ctx_write_async(strategy_id, |ctx| {
    ///     Box::pin(async move {
    ///         ctx.update_something().await
    ///     })
    /// }).await?;
    /// ```
    pub async fn with_strategy_ctx_write_async<R>(
        &self,
        strategy_id: StrategyId,
        f: impl for<'a> FnOnce(&'a mut BacktestStrategyContext) -> std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'a>> + Send + 'static
    ) -> Result<R, crate::backtest_engine_error::BacktestEngineError>
    where
        R: Send,
    {
        self.with_strategy_mut_async(strategy_id, |strategy| {
            Box::pin(async move {
                strategy.with_ctx_write_async(f).await
            })
        }).await
    }


}


impl EngineContextTrait for BacktestEngineContext {
    type Action = BacktestEngineAction;

    fn base_context(&self) -> &EngineBaseContext<BacktestEngineAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineBaseContext<BacktestEngineAction> {
        &mut self.base_context
    }

}