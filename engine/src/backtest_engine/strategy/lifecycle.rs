use super::BacktestStrategy;
use super::strategy_state_machine::{StrategyRunState, StrategyStateTransEvent};
use star_river_core::error::strategy_error::backtest_strategy_error::{BacktestStrategyError, WaitAllNodesStoppedTimeoutSnafu};

impl BacktestStrategy {
    pub async fn check_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.with_ctx_read(|ctx| ctx.strategy_name().clone()).await;

        tracing::info!("[{}] starting check strategy", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.update_strategy_status(StrategyRunState::Checking.to_string().to_lowercase()).await
            })
        }).await?;

        let update_result = self.update_strategy_state(StrategyStateTransEvent::Check).await;
        
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.update_strategy_status(StrategyRunState::Failed.to_string().to_lowercase()).await
                })
            }).await?;
            return Err(e);
        }

        tracing::info!("[{}] check finished.", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.update_strategy_status(StrategyRunState::CheckPassed.to_string().to_lowercase()).await
            })
        }).await?;


        let update_result = self
            .update_strategy_state(StrategyStateTransEvent::CheckComplete)
            .await;

        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.update_strategy_status(StrategyRunState::Failed.to_string().to_lowercase()).await
                })
            }).await?;
            return Err(e);
        }
        Ok(())
    }

    pub async fn init_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.with_ctx_read(|ctx| ctx.strategy_name().clone()).await;
        tracing::info!("[{}] starting init strategy", strategy_name);

        // created => initializing
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.update_strategy_status(StrategyRunState::Initializing.to_string().to_lowercase()).await
            })
        }).await?;


        let update_result = self.update_strategy_state(StrategyStateTransEvent::Initialize).await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.update_strategy_status(StrategyRunState::Failed.to_string().to_lowercase()).await
                })
            }).await?;
            return Err(e);
        }

        //
        // initializing => ready
        tracing::info!("[{}] init finished.", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.update_strategy_status(StrategyRunState::Ready.to_string().to_lowercase()).await
            })
        }).await?;
        let update_result = self
            .update_strategy_state(StrategyStateTransEvent::InitializeComplete)
            .await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.update_strategy_status(StrategyRunState::Failed.to_string().to_lowercase()).await
                })
            }).await?;
            return Err(e);
        }

        Ok(())
    }

    pub async fn stop_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let (strategy_name, current_state) = self
            .with_ctx_read(|ctx| (ctx.strategy_name().clone(), ctx.state_machine().current_state()))
            .await;
        if current_state == StrategyRunState::Stopping {
            tracing::info!("[{}] stopped.", strategy_name);
            return Ok(());
        }
        tracing::info!("waiting for all nodes to stop...");
        self.context
            .write()
            .await
            .update_strategy_status(StrategyRunState::Stopping.to_string().to_lowercase())
            .await?;
        let update_result = self.update_strategy_state(StrategyStateTransEvent::Stop).await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.update_strategy_status(StrategyRunState::Failed.to_string().to_lowercase()).await
                })
            }).await?;
            return Err(e);
        }

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = self
            .with_ctx_read_async(|ctx| Box::pin(ctx.wait_for_all_nodes_stopped(10)))
            .await
            .unwrap();

        if all_stopped {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.update_strategy_status(StrategyRunState::Stopped.to_string().to_lowercase()).await
                })
            }).await?;
            let update_result = self.update_strategy_state(StrategyStateTransEvent::StopComplete).await;
            if let Err(e) = update_result {
                self.with_ctx_write_async(|ctx| {
                    Box::pin(async move {
                        ctx.update_strategy_status(StrategyRunState::Failed.to_string().to_lowercase()).await
                    })
                }).await?;
                return Err(e);
            }
            Ok(())
        } else {
            Err(WaitAllNodesStoppedTimeoutSnafu {}.build())
        }
    }
}
