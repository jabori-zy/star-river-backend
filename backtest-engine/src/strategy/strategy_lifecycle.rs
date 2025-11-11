use chrono::Utc;
use event_center::EventCenterSingleton;
use star_river_core::error::{ErrorLanguage, StarRiverErrorTrait};
use star_river_event::backtest_strategy::strategy_event::BacktestStrategyEvent;
use strategy_core::{
    error::strategy_error::WaitAllNodesStoppedTimeoutSnafu,
    event::{log_event::LogLevel, strategy_event::StrategyStateLogEvent},
    strategy::{
        context_trait::{StrategyIdentityExt, StrategyInfraExt, StrategyStateMachineExt, StrategyWorkflowExt},
        state_machine::{StrategyRunState, StrategyStateMachine, StrategyStateTransTrigger},
        strategy_trait::{StrategyContextAccessor, StrategyEventListener, StrategyLifecycle},
    },
};

use super::{
    BacktestStrategy,
    strategy_state_machine::{BacktestStrategyRunState, BacktestStrategyStateAction, BacktestStrategyStateTransTrigger},
};
use crate::strategy::{strategy_error::BacktestStrategyError, strategy_log_message::StrategyStateLogMsg};

#[async_trait::async_trait]
impl StrategyLifecycle for BacktestStrategy {
    type Trigger = BacktestStrategyStateTransTrigger;
    type Error = BacktestStrategyError;

    async fn init_strategy(&mut self) -> Result<(), Self::Error> {
        let strategy_name = self.with_ctx_read(|ctx| ctx.strategy_name().clone()).await;
        tracing::info!("[{}] starting init strategy", strategy_name);

        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.store_strategy_status(BacktestStrategyRunState::Initializing.to_string().to_lowercase())
                    .await
            })
        })
        .await?;

        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::Initialize).await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.store_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                        .await
                })
            })
            .await?;
            return Err(e);
        }

        //
        // initializing => ready
        tracing::info!("[{}] init finished.", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.store_strategy_status(BacktestStrategyRunState::Ready.to_string().to_lowercase())
                    .await
            })
        })
        .await?;
        let update_result = self
            .update_strategy_state(BacktestStrategyStateTransTrigger::InitializeComplete)
            .await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.store_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                        .await
                })
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
            Box::pin(async move {
                ctx.store_strategy_status(BacktestStrategyRunState::Stopping.to_string().to_lowercase())
                    .await
            })
        })
        .await?;
        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::Stop).await;
        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.store_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                        .await
                })
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
                    ctx.store_strategy_status(BacktestStrategyRunState::Stopped.to_string().to_lowercase())
                        .await
                })
            })
            .await?;

            let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::StopComplete).await;
            if let Err(e) = update_result {
                self.with_ctx_write_async(|ctx| {
                    Box::pin(async move {
                        ctx.store_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                            .await
                    })
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
        let (strategy_name, state_machine) = self
            .with_ctx_read(|ctx| {
                let strategy_name = ctx.strategy_name().clone();
                let state_machine = ctx.state_machine().clone();
                (strategy_name, state_machine)
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
                            let start_node_config = ctx.get_start_node_config().await;
                            if let Ok(start_node_config) = start_node_config {
                                ctx.set_initial_play_speed(start_node_config.play_speed as u32).await;
                                tracing::info!(
                                    "[{}] init initial play speed success. initial play speed: {:?}",
                                    &strategy_name_clone,
                                    ctx.initial_play_speed().await
                                );
                            } else {
                                tracing::error!("[{}] get start node config failed", &strategy_name_clone);
                            }
                        })
                    })
                    .await;
                }
                BacktestStrategyStateAction::InitSignalCount => {
                    let strategy_name_clone = strategy_name.clone();
                    self.with_ctx_write_async(|ctx| {
                        Box::pin(async move {
                            let signal_count = ctx.get_signal_count().await;
                            if let Ok(count) = signal_count {
                                ctx.set_total_signal_count(count).await;
                            } else {
                                tracing::error!("[{}] get signal count failed", &strategy_name_clone);
                            }
                        })
                    })
                    .await;
                }
                BacktestStrategyStateAction::InitVirtualTradingSystem => {
                    // let strategy_name_clone = strategy_name.clone();
                    // self.with_ctx_write_async(|ctx| {
                    //     Box::pin(async move {
                    //         if let Err(e) = VirtualTradingSystem::listen_play_index(ctx.virtual_trading_system().clone()).await {
                    //             tracing::error!("[{}] init virtual trading system failed: {}", &strategy_name_clone, e);
                    //         } else {
                    //             tracing::info!("[{}] init virtual trading system success", &strategy_name_clone);
                    //         }
                    //     })
                    // }).await;
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

                BacktestStrategyStateAction::CheckNode => {
                    tracing::info!("[{}] start check node", &strategy_name);
                    let (strategy_id, current_state) = self
                        .with_ctx_read_async(|ctx| Box::pin(async move { (ctx.strategy_id(), ctx.run_state().await) }))
                        .await;

                    // 辅助函数：处理检查步骤错误
                    let strategy_name_for_error = strategy_name.clone();
                    let handle_error = |e: BacktestStrategyError| -> BacktestStrategyError {
                        let error_message = e.error_message(ErrorLanguage::Chinese);
                        let log_event: BacktestStrategyEvent = StrategyStateLogEvent::new(
                            strategy_id,
                            strategy_name_for_error.clone(),
                            Some(current_state.to_string()),
                            Some(BacktestStrategyStateAction::CheckNode.to_string()),
                            LogLevel::Error,
                            Some(e.error_code()),
                            Some(e.error_code_chain()),
                            error_message,
                        )
                        .into();

                        // let backtest_strategy_event = BacktestStrategyEvent::StrategyStateLog(log_event);
                        let _ = futures::executor::block_on(EventCenterSingleton::publish(log_event.into()));
                        e
                    };

                    // 按顺序执行检查步骤
                    if let Err(e) = self.with_ctx_write_async(|ctx| Box::pin(ctx.build_workflow())).await {
                        return Err(handle_error(e));
                    }
                    // if let Err(e) = self.add_edge().await {
                    //     return Err(handle_error(e));
                    // }
                    // if let Err(e) = self.check_symbol_config().await {
                    //     return Err(handle_error(e));
                    // }
                    // if let Err(e) = self.set_leaf_nodes().await {
                    //     return Err(handle_error(e));
                    // }
                    // if let Err(e) = self.set_strategy_output_handles().await {
                    //     return Err(handle_error(e));
                    // }
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

                    let log_message = StrategyStateLogMsg::new(strategy_id, strategy_name.clone(), current_state.to_string());
                    let log_event = StrategyStateLogEvent {
                        strategy_id,
                        strategy_name: strategy_name.clone(),
                        strategy_state: Some(current_state.to_string()),
                        strategy_state_action: Some(BacktestStrategyStateAction::LogStrategyState.to_string()),
                        log_level: LogLevel::Info,
                        error_code: None,
                        error_code_chain: None,
                        message: log_message.to_string(),
                        datetime: Utc::now(),
                    };
                    let backtest_strategy_event = BacktestStrategyEvent::StrategyStateLog(log_event.clone());
                    let _ = EventCenterSingleton::publish(backtest_strategy_event.into()).await;
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
            Box::pin(async move {
                ctx.store_strategy_status(BacktestStrategyRunState::Checking.to_string().to_lowercase())
                    .await
            })
        })
        .await;

        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::Check).await;

        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.store_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                        .await
                })
            })
            .await;
            return Err(e);
        }

        tracing::info!("[{}] check finished.", strategy_name);
        self.with_ctx_write_async(|ctx| {
            Box::pin(async move {
                ctx.store_strategy_status(BacktestStrategyRunState::CheckPassed.to_string().to_lowercase())
                    .await
            })
        })
        .await;

        let update_result = self.update_strategy_state(BacktestStrategyStateTransTrigger::CheckComplete).await;

        if let Err(e) = update_result {
            self.with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    ctx.store_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase())
                        .await
                })
            })
            .await;
            return Err(e);
        }
        Ok(())
    }
}
