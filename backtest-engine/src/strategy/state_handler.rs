use super::{
    BacktestStrategy,
    strategy_state_machine::{BacktestStrategyStateAction, StrategyStateTransTrigger},
};
// use event_center::{
//     EventCenterSingleton,
//     event::strategy_event::{
//         LogLevel,
//         backtest_strategy_event::{BacktestStrategyEvent, StrategyStateLogEvent},
//     },
// };
use crate::error::strategy_error::BacktestStrategyError;
// use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use virtual_trading::VirtualTradingSystem;
use crate::strategy::strategy_context::BacktestStrategyContext;
use crate::strategy::strategy_log_message::StrategyStateLogMsg;
use chrono::Utc;
use star_river_core::error::{ErrorLanguage, StarRiverErrorTrait};
use star_river_event::backtest_strategy::strategy_event::{StrategyStateLogEvent, log_event::LogLevel};
use event_center::EventCenterSingleton;
use star_river_event::backtest_strategy::strategy_event::BacktestStrategyEvent;

impl BacktestStrategy {
    pub async fn update_strategy_state(&mut self, event: StrategyStateTransTrigger) -> Result<(), BacktestStrategyError> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let (strategy_name, mut state_machine) = self.with_ctx_read(|ctx| {
            let strategy_name = ctx.strategy_name().clone();
            let state_machine = ctx.state_machine().clone();
            (strategy_name, state_machine)
        }).await;

        // 在这里调用 transition,这样可以直接使用 ? 操作符
        let transition_result = state_machine.transition(event)?;


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
                    }).await;
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
                    }).await;
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
                        .with_ctx_read(|ctx| (ctx.strategy_id(), ctx.state_machine().current_state()))
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
                        ).into();

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
                    BacktestStrategyContext::init_node(self.context.clone()).await.unwrap();

                    tracing::info!("[{}] all nodes initialized.", &strategy_name);
                }

                BacktestStrategyStateAction::StopNode => {
                    tracing::info!("[{}] start stop node", &strategy_name);
                    // 传入 context 引用，让 stop_node 方法内部控制锁的生命周期
                    BacktestStrategyContext::stop_node(self.context.clone()).await.unwrap();
                    tracing::info!("[{}] all nodes stopped", &strategy_name);
                }

                BacktestStrategyStateAction::LogTransition => {
                    let current_state = self.with_ctx_read(|ctx| ctx.state_machine().current_state()).await;
                    tracing::debug!(
                        "[{}] state transition: {:?} -> {:?}",
                        &strategy_name,
                        current_state,
                        transition_result.get_new_state()
                    );
                }

                BacktestStrategyStateAction::ListenAndHandleNodeEvent => {
                    tracing::info!("[{}] listen node events", &strategy_name);
                    self.listen_node_events().await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{}] listen strategy command", &strategy_name);
                    self.listen_strategy_command().await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent => {
                    tracing::info!("[{}] listen strategy stats event", &strategy_name);
                    self.listen_strategy_stats_event().await;
                }
                BacktestStrategyStateAction::LogError(error) => {
                    tracing::error!("[{}] {}", &strategy_name, error);
                }
                BacktestStrategyStateAction::LogStrategyState => {
                    let (strategy_id, current_state) = self.with_ctx_read(|ctx| (ctx.strategy_id(), ctx.state_machine().current_state())).await;

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

            {
                let mut context_guard = self.context.write().await;
                context_guard.set_state_machine(state_machine.clone());
            }
        }
        Ok(())
    }
}
