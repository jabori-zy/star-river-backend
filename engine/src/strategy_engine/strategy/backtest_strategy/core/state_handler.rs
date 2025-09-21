use super::super::super::backtest_strategy::BacktestStrategy;
use crate::strategy_engine::strategy::backtest_strategy::*;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyStateTransitionEvent;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyError;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyStateAction;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyStats;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyContext;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyFunction;
use crate::strategy_engine::strategy::backtest_strategy::BacktestStrategyEvent;
use snafu::Report;
use star_river_core::error::error_trait::Language;


impl BacktestStrategy {
    pub async fn update_strategy_state(
        &mut self,
        event: BacktestStrategyStateTransitionEvent,
    ) -> Result<(), BacktestStrategyError> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;

        let (transition_result, state_machine) = {
            let mut state_manager = self.context.read().await.state_machine.clone();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        for action in transition_result.get_actions() {
            // action execute result flag
            match action {
                BacktestStrategyStateAction::InitInitialPlaySpeed => {
                    tracing::info!(
                        "[{}({})] init initial play speed",
                        strategy_name,
                        strategy_id
                    );
                    let context_guard = self.context.read().await;
                    let start_node_config = context_guard.get_start_node_config().await;
                    if let Ok(start_node_config) = start_node_config {
                        let mut initial_play_speed_guard =
                            context_guard.initial_play_speed.write().await;
                        *initial_play_speed_guard = start_node_config.play_speed as u32;
                        tracing::info!(
                            "[{}({})] init initial play speed success. initial play speed: {:?}",
                            strategy_name,
                            strategy_id,
                            *initial_play_speed_guard
                        );
                    } else {
                        tracing::error!(
                            "[{}({})] get start node config failed",
                            strategy_name,
                            strategy_id
                        );
                    }
                }
                BacktestStrategyStateAction::InitSignalCount => {
                    tracing::info!("[{}({})] init signal count", strategy_name, strategy_id);
                    let mut context_guard = self.context.write().await;
                    let signal_count = context_guard.get_signal_count().await;


                    if let Ok(signal_count) = signal_count {
                        let mut signal_count_guard = context_guard.total_signal_count.write().await;
                        *signal_count_guard = signal_count;
                        tracing::info!(
                            "[{}({})] init signal count success",
                            strategy_name,
                            strategy_id
                        );
                    } else {
                        tracing::error!(
                            "[{}({})] get signal count failed",
                            strategy_name,
                            strategy_id
                        );
                    }
                }
                BacktestStrategyStateAction::InitVirtualTradingSystem => {
                    tracing::info!(
                        "[{}({})] init virtual trading system",
                        strategy_name,
                        strategy_id
                    );
                    let context_guard = self.context.read().await;
                    let virtual_trading_system = context_guard.virtual_trading_system.clone();
                    drop(context_guard); // 释放锁
                    if let Err(e) =
                        VirtualTradingSystem::listen_play_index(virtual_trading_system).await
                    {
                        tracing::error!(
                            "[{}({})] init virtual trading system failed: {}",
                            strategy_name,
                            strategy_id,
                            e
                        );
                    } else {
                        tracing::info!(
                            "[{}({})] init virtual trading system success",
                            strategy_name,
                            strategy_id
                        );
                    }
                }
                BacktestStrategyStateAction::InitStrategyStats => {
                    tracing::info!("[{}({})] init strategy stats", strategy_name, strategy_id);
                    let context_guard = self.context.read().await;
                    let strategy_stats = context_guard.strategy_stats.clone();
                    drop(context_guard); // 释放锁

                    if let Err(e) =
                        BacktestStrategyStats::handle_virtual_trading_system_events(strategy_stats)
                            .await
                    {
                        tracing::error!(
                            "[{}({})] init strategy stats failed: {}",
                            strategy_name,
                            strategy_id,
                            e
                        );
                    } else {
                        tracing::info!(
                            "[{}({})] init strategy stats success",
                            strategy_name,
                            strategy_id
                        );
                    }
                }

                BacktestStrategyStateAction::CheckNode => {
                    let (strategy_id, strategy_name, current_state) = {
                        let context_guard = self.context.read().await;
                        (
                            context_guard.strategy_id,
                            context_guard.strategy_name.clone(),
                            self.get_state_machine().await.current_state(),
                        )
                    };

                    // 辅助函数：处理检查步骤错误
                    let handle_error = |e: BacktestStrategyError| -> BacktestStrategyError {
                        let error_message = e.get_error_message(Language::Chinese);
                        let log_event = StrategyStateLogEvent::new(
                            strategy_id,
                            strategy_name.clone(),
                            Some(current_state.to_string()),
                            Some(BacktestStrategyStateAction::CheckNode.to_string()),
                            LogLevel::Error,
                            Some(e.error_code()),
                            Some(e.error_code_chain()),
                            error_message,
                        );

                        let backtest_strategy_event = BacktestStrategyEvent::StrategyStateLog(log_event);
                        let _ = futures::executor::block_on(
                            EventCenterSingleton::publish(backtest_strategy_event.into())
                        );
                        e
                    };

                    // 按顺序执行检查步骤
                    if let Err(e) = self.add_node().await {
                        return Err(handle_error(e));
                    }
                    if let Err(e) = self.add_edge().await {
                        return Err(handle_error(e));
                    }
                    if let Err(e) = self.check_symbol_config().await {
                        return Err(handle_error(e));
                    }
                    if let Err(e) = self.set_leaf_nodes().await {
                        return Err(handle_error(e));
                    }
                    if let Err(e) = self.set_strategy_output_handles().await {
                        return Err(handle_error(e));
                    }
                }

                BacktestStrategyStateAction::InitNode => {
                    let strategy_id = self.get_strategy_id().await;
                    tracing::info!("[{}({})] start init node", strategy_name, strategy_id);

                    // business logic is in context, here only to get the lock
                    if let Err(e) = BacktestStrategyContext::init_node(self.context.clone()).await {
                        tracing::error!("{}", Report::from_error(&e));
                        return Err(e);
                    }

                    tracing::info!("[{}] all nodes initialized.", strategy_name);
                }

                BacktestStrategyStateAction::StopNode => {
                    tracing::info!("[{}] start stop node", strategy_name);
                    let nodes = {
                        let context_guard = self.context.read().await;
                        context_guard.topological_sort()
                    };

                    let mut all_nodes_stopped = true;

                    for node in nodes {
                        // let mut node = node.clone();
                        let context_guard = self.context.read().await;

                        if let Err(e) = context_guard.stop_node(node).await {
                            tracing::error!("{}", e);
                            all_nodes_stopped = false;
                            break;
                        }
                    }

                    if all_nodes_stopped {
                        tracing::info!("[{}] all nodes stopped", strategy_name);
                    } else {
                        tracing::error!(
                            "[{}] some nodes stop failed, strategy cannot run normally",
                            strategy_name
                        );
                    }
                }

                BacktestStrategyStateAction::LogTransition => {
                    tracing::debug!(
                        "[{}] state transition: {:?} -> {:?}",
                        strategy_name,
                        self.get_state_machine().await.current_state(),
                        transition_result.get_new_state()
                    );
                }

                BacktestStrategyStateAction::ListenAndHandleNodeEvent => {
                    tracing::info!("[{}] listen node events", strategy_name);
                    BacktestStrategyFunction::listen_node_events(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleNodeCommand => {
                    tracing::info!("[{}] listen node command", strategy_name);
                    BacktestStrategyFunction::listen_node_command(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent => {
                    tracing::info!("[{}] listen strategy stats event", strategy_name);
                    BacktestStrategyFunction::listen_strategy_stats_event(self.get_context()).await;
                }
                BacktestStrategyStateAction::LogError(error) => {
                    tracing::error!("[{}] {}", strategy_name, error);
                }
                BacktestStrategyStateAction::LogStrategyState => {
                    let current_state = self.get_state_machine().await.current_state();

                    let log_message = StrategyStateLogMsg::new(
                        strategy_id,
                        strategy_name.clone(),
                        current_state.to_string(),
                    );
                    let log_event = StrategyStateLogEvent {
                        strategy_id,
                        strategy_name: strategy_name.clone(),
                        strategy_state: Some(current_state.to_string()),
                        strategy_state_action: Some(
                            BacktestStrategyStateAction::LogStrategyState.to_string(),
                        ),
                        log_level: LogLevel::Info,
                        error_code: None,
                        error_code_chain: None,
                        message: log_message.to_string(),
                        datetime: Utc::now(),
                    };
                    let backtest_strategy_event =
                        BacktestStrategyEvent::StrategyStateLog(log_event.clone());
                    // let _ = self.get_context().read().await.get_event_publisher().publish(backtest_strategy_event.into()).await;
                    let _ = EventCenterSingleton::publish(backtest_strategy_event.into()).await;
                }
            };

            {
                let mut context_guard = self.context.write().await;
                context_guard.set_state_machine(state_machine.clone());
            }
        }
        Ok(())
    }
}