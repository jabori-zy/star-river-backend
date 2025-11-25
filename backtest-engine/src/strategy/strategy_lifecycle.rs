use std::time::Duration;

use event_center::EventCenterSingleton;
use star_river_event::backtest_strategy::strategy_event::BacktestStrategyEvent;
use strategy_core::{
    error::strategy_error::WaitAllNodesStoppedTimeoutSnafu,
    event::strategy_event::StrategyStateLogEvent,
    strategy::{
        context_trait::{StrategyIdentityExt, StrategyStateMachineExt, StrategyWorkflowExt},
        state_machine::StrategyStateMachine,
        strategy_trait::{StrategyContextAccessor, StrategyEventListener, StrategyLifecycle},
    },
};
use virtual_trading::vts_trait::VtsCtxAccessor;

use super::{
    BacktestStrategy,
    strategy_state_machine::{BacktestStrategyRunState, BacktestStrategyStateAction, BacktestStrategyStateTransTrigger},
};
use crate::strategy::{
    strategy_error::{BacktestStrategyError, TimeRangeNotConfiguredSnafu},
    strategy_log_message::StrategyStateLogMsg,
};

#[async_trait::async_trait]
impl StrategyLifecycle for BacktestStrategy {
    type Trigger = BacktestStrategyStateTransTrigger;
    type Error = BacktestStrategyError;

    async fn init_strategy(&mut self) -> Result<(), Self::Error> {
        let strategy_name = self.with_ctx_read(|ctx| ctx.strategy_name().clone()).await;
        tracing::info!("[{}] starting init strategy", strategy_name);

        self.with_ctx_write_async(|ctx| {
            Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Initializing.to_string()).await })
        })
        .await?;

        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::Initialize).await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Error.to_string()).await })
            })
            .await?;
            return Err(e);
        }

        //
        // initializing => ready
        tracing::info!("[{}] init finished.", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Ready.to_string()).await })
        })
        .await?;
        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransTrigger::InitializeComplete)
            .await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Error.to_string()).await })
            })
            .await?;
            return Err(e);
        }

        Ok(())
    }

    async fn stop_strategy(&mut self) -> Result<(), Self::Error> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let (strategy_name, current_state) = self
            .with_ctx_read_async(|ctx| Box::pin(async move { (ctx.strategy_name().clone(), ctx.run_state().await) }))
            .await;
        if current_state == BacktestStrategyRunState::Stopping {
            tracing::info!("[{}] stopped.", strategy_name);
            return Ok(());
        }
        tracing::info!("waiting for all nodes to stop...");
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Stopping.to_string()).await })
        })
        .await?;
        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::Stop).await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Error.to_string()).await })
            })
            .await?;
            return Err(e);
        }

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = self
            .with_ctx_read_async(|ctx| Box::pin(ctx.wait_for_all_nodes_stopped(20)))
            .await
            .unwrap();

        if all_stopped {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    tracing::debug!("store strategy status to stopped");
                    ctx.store_strategy_status(BacktestStrategyRunState::Stopped.to_string()).await
                })
            })
            .await?;

            let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::StopComplete).await;
            if let Err(e) = update_result {
                self.with_ctx_write_async(|ctx| {
                    Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Error.to_string()).await })
                })
                .await?;
                return Err(e);
            }
            Ok(())
        } else {
            Err(WaitAllNodesStoppedTimeoutSnafu {}.build().into())
        }
    }

    async fn update_strategy_state(&mut self, trigger: Self::Trigger) -> Result<(), Self::Error> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let (strategy_id, strategy_name, state_machine) = self
            .with_ctx_read(|ctx| {
                let strategy_id = ctx.strategy_id();
                let strategy_name = ctx.strategy_name().clone();
                let state_machine = ctx.state_machine().clone();
                (strategy_id, strategy_name, state_machine)
            })
            .await;

        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(trigger)?
        };

        for action in transition_result.get_actions() {
            // action execute result flag
            match action {
                BacktestStrategyStateAction::InitInitialPlaySpeed => {
                    tracing::info!("[{}] init initial play speed", &strategy_name);
                    let strategy_name_clone = strategy_name.clone();
                    self.with_ctx_write_async(|ctx| {
                        Box::pin(async move {
                            let strategy_config = ctx.get_strategy_config().await?;
                            ctx.set_initial_play_speed(strategy_config.play_speed as u32).await;
                            tracing::info!(
                                "[{}] init initial play speed success. initial play speed: {:?}",
                                &strategy_name_clone,
                                ctx.initial_play_speed().await
                            );
                            Ok::<(), BacktestStrategyError>(())
                        })
                    })
                    .await?;
                }
                BacktestStrategyStateAction::InitSignalGenerator => {
                    self.with_ctx_write_async(|ctx| {
                        Box::pin(async move {
                            let strategy_config = ctx.get_strategy_config().await?;
                            let start_time = strategy_config.start_time();
                            let end_time = strategy_config.end_time();
                            if let (Some(start_time), Some(end_time)) = (start_time, end_time) {
                                ctx.signal_generator
                                    .lock()
                                    .await
                                    .init(start_time, end_time, ctx.min_interval().clone());
                                Ok::<(), BacktestStrategyError>(())
                            } else {
                                return Err(TimeRangeNotConfiguredSnafu {
                                    strategy_name: ctx.strategy_name().clone(),
                                }
                                .build());
                            }
                        })
                    })
                    .await?;
                }
                BacktestStrategyStateAction::InitVirtualTradingSystem => {
                    self.with_ctx_write_async(|ctx| {
                        Box::pin(async move {
                            let strategy_config = ctx.get_strategy_config().await?;
                            {
                                let vts_guard = ctx.virtual_trading_system().lock().await;
                                vts_guard
                                    .with_ctx_write(|ctx| {
                                        ctx.set_initial_balance(strategy_config.initial_balance);
                                        ctx.set_leverage(strategy_config.leverage as u32);
                                        ctx.set_fee_rate(strategy_config.fee_rate);
                                    })
                                    .await;
                                vts_guard.start().await;
                            }
                            tracing::info!("[{}] init virtual trading system success", ctx.strategy_name());
                            Ok::<(), BacktestStrategyError>(())
                        })
                    })
                    .await?;
                }
                BacktestStrategyStateAction::InitStrategyStats => {
                    // let strategy_name_clone = strategy_name.clone();
                    // self.with_ctx_write_async(|ctx| {
                    //     Box::pin(async move {
                    //         if let Err(e) = BacktestStrategyStats::handle_virtual_trading_system_events(ctx.strategy_stats()).await {
                    //             tracing::error!("[{}] init strategy stats failed: {}", &strategy_name_clone, e);
                    //         } else {
                    //             tracing::info!("[{}] init strategy stats success", &strategy_name_clone);
                    //         }
                    //     })
                    // }).await;
                }

                BacktestStrategyStateAction::StoreStrategyStatus => {
                    let current_state = state_machine.read().await.current_state().clone();
                    self.with_ctx_write_async(|ctx| Box::pin(async move { ctx.store_strategy_status(current_state.to_string()).await }))
                        .await?;
                }

                BacktestStrategyStateAction::CheckNode => {
                    // 按顺序执行检查步骤
                    let build_result = self.with_ctx_write_async(|ctx| Box::pin(ctx.build_workflow())).await;
                    if let Err(e) = build_result {
                        let log_event: BacktestStrategyEvent = StrategyStateLogEvent::error(
                            strategy_id,
                            strategy_name.clone(),
                            BacktestStrategyRunState::Error.to_string(),
                            BacktestStrategyStateAction::CheckNode.to_string(),
                            &e,
                        )
                        .into();
                        // sleep 500 milliseconds
                        tokio::time::sleep(Duration::from_millis(2000)).await;
                        let _ = EventCenterSingleton::publish(log_event.into()).await;
                        return Err(e);
                    }
                }

                BacktestStrategyStateAction::InitNode => {
                    tracing::info!("[{}] start init node", &strategy_name);

                    // 传入 context 引用，让 init_node 方法内部控制锁的生命周期
                    StrategyWorkflowExt::init_node(self.context.clone()).await.unwrap();

                    tracing::info!("#[{}] all nodes initialized.", &strategy_name);
                }

                BacktestStrategyStateAction::StopNode => {
                    tracing::info!("#[{}] start stop node", &strategy_name);
                    // 传入 context 引用，让 stop_node 方法内部控制锁的生命周期
                    StrategyWorkflowExt::stop_node(self.context.clone()).await.unwrap();
                }

                BacktestStrategyStateAction::LogTransition => {
                    let (previous_state, current_state) = {
                        let state_machine = state_machine.read().await;
                        (state_machine.previous_state().clone(), state_machine.current_state().clone())
                    };
                    tracing::debug!(
                        "#[{}] state transition: {:?} -> {:?}",
                        &strategy_name,
                        previous_state,
                        current_state
                    );
                }

                BacktestStrategyStateAction::ListenAndHandleNodeEvent => {
                    tracing::info!("#[{}] listen node events", &strategy_name);
                    self.listen_node_events().await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("#[{}] listen strategy command", &strategy_name);
                    self.listen_strategy_command().await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent => {
                    tracing::info!("#[{}] listen strategy stats event", &strategy_name);
                    // self.listen_strategy_stats_event().await;
                }
                BacktestStrategyStateAction::LogError(error) => {
                    tracing::error!("#[{}] {}", &strategy_name, error);
                }
                BacktestStrategyStateAction::LogStrategyState => {
                    let (strategy_id, current_state) = self
                        .with_ctx_read_async(|ctx| Box::pin(async move { (ctx.strategy_id(), ctx.run_state().await) }))
                        .await;

                    let log_message = StrategyStateLogMsg::new(strategy_name.clone(), current_state.to_string());
                    let log_event: BacktestStrategyEvent = StrategyStateLogEvent::info(
                        strategy_id,
                        strategy_name.clone(),
                        current_state.to_string(),
                        BacktestStrategyStateAction::LogStrategyState.to_string(),
                        log_message.to_string(),
                    )
                    .into();
                    // sleep 500 milliseconds
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let _ = EventCenterSingleton::publish(log_event.into()).await;
                }
            };
        }
        Ok(())
    }
}

impl BacktestStrategy {
    pub async fn check_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.with_ctx_read(|ctx| ctx.strategy_name().clone()).await;

        tracing::info!("[{}] starting check strategy", strategy_name);

        self.with_ctx_write_async(|ctx| {
            Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Checking.to_string()).await })
        })
        .await?;

        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::Check).await;

        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Error.to_string()).await })
            })
            .await?;
            return Err(e);
        }

        tracing::info!("[{}] check finished.", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::CheckPassed.to_string()).await })
        })
        .await?;

        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::CheckComplete).await;

        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move { ctx.store_strategy_status(BacktestStrategyRunState::Error.to_string()).await })
            })
            .await?;
            return Err(e);
        }
        Ok(())
    }
}
