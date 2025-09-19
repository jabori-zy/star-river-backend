use super::BacktestStrategy;
use crate::strategy_engine::strategy::backtest_strategy::*;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyRunState;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyStateTransitionEvent;



impl BacktestStrategy {
    pub async fn check_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;

        tracing::info!(
            "[{}({})] starting check strategy",
            strategy_name,
            strategy_id
        );
        self.context
            .write()
            .await
            .update_strategy_status(
                BacktestStrategyRunState::Checking
                    .to_string()
                    .to_lowercase(),
            )
            .await?;

        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransitionEvent::Check)
            .await;
        if let Err(e) = update_result {
            self.context
                .write()
                .await
                .update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                .await?;
            return Err(e);
        }

        tracing::info!("[{}({})] check finished.", strategy_name, strategy_id);
        self.context
            .write()
            .await
            .update_strategy_status(
                BacktestStrategyRunState::CheckPassed
                    .to_string()
                    .to_lowercase(),
            )
            .await?;
        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransitionEvent::CheckComplete)
            .await;
        if let Err(e) = update_result {
            self.context
                .write()
                .await
                .update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                .await?;
            return Err(e);
        }
        Ok(())
    }

    pub async fn init_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        tracing::info!(
            "[{}({})] starting init strategy",
            strategy_name,
            strategy_id
        );

        // created => initializing
        self.context
            .write()
            .await
            .update_strategy_status(
                BacktestStrategyRunState::Initializing
                    .to_string()
                    .to_lowercase(),
            )
            .await?;
        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransitionEvent::Initialize)
            .await;
        if let Err(e) = update_result {
            self.context
                .write()
                .await
                .update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                .await?;
            return Err(e);
        }

        //
        // initializing => ready
        tracing::info!("[{}({})] init finished.", strategy_name, strategy_id);
        self.context
            .write()
            .await
            .update_strategy_status(BacktestStrategyRunState::Ready.to_string().to_lowercase())
            .await?;
        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransitionEvent::InitializeComplete)
            .await;
        if let Err(e) = update_result {
            self.context
                .write()
                .await
                .update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                .await?;
            return Err(e);
        }

        Ok(())
    }

    pub async fn stop_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let current_state = self.get_state_machine().await.current_state();
        if current_state == BacktestStrategyRunState::Stopping {
            tracing::info!(
                "[{}({})] stopped.",
                self.get_strategy_name().await,
                self.get_strategy_id().await
            );
            return Ok(());
        }
        tracing::info!("waiting for all nodes to stop...");
        self.context
            .write()
            .await
            .update_strategy_status(
                BacktestStrategyRunState::Stopping
                    .to_string()
                    .to_lowercase(),
            )
            .await?;
        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransitionEvent::Stop)
            .await;
        if let Err(e) = update_result {
            self.context
                .write()
                .await
                .update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                .await?;
            return Err(e);
        }

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = {
            let context_guard = self.context.read().await;
            context_guard.wait_for_all_nodes_stopped(10).await.unwrap()
        };
        if all_stopped {
            self.context
                .write()
                .await
                .update_strategy_status(
                    BacktestStrategyRunState::Stopped.to_string().to_lowercase(),
                )
                .await?;
            let update_result = self
                .update_strategy_state(BacktestStrategyStateTransitionEvent::StopComplete)
                .await;
            if let Err(e) = update_result {
                self.context
                    .write()
                    .await
                    .update_strategy_status(
                        BacktestStrategyRunState::Failed.to_string().to_lowercase(),
                    )
                    .await?;
                return Err(e);
            }
            Ok(())
        } else {
            Err(WaitAllNodesStoppedTimeoutSnafu {}.build())
        }
    }
}