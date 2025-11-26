use async_trait::async_trait;
use star_river_core::kline::KlineInterval;
use strategy_core::{
    NodeType,
    node::{
        context_trait::{NodeHandleExt, NodeInfoExt, NodeStateMachineExt, NodeTaskControlExt},
        node_state_machine::StateMachine,
        node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
    },
};

use super::IndicatorNode;
use crate::{
    node::{
        node_error::{BacktestNodeError, IndicatorNodeError},
        node_message::{
            common_log_message::{ListenExternalEventsMsg, ListenNodeEventsMsg, ListenStrategyCommandMsg, NodeStateLogMsg},
            indicator_node_log_message::{
                CalculateIndicatorMsg, CalculateIndicatorSuccessMsg, InitLookbackSuccessMsg, InitMinIntervalSuccessMsg,
            },
        },
        node_state_machine::NodeStateTransTrigger,
        node_utils::NodeUtils,
    },
    node_catalog::indicator_node::state_machine::IndicatorNodeAction,
};

#[async_trait]
impl NodeLifecycle for IndicatorNode {
    type Error = BacktestNodeError;

    type Trigger = NodeStateTransTrigger;

    async fn init(&self) -> Result<(), Self::Error> {
        NodeUtils::init_node(self, None).await
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        NodeUtils::stop_node(self, Some(500)).await
    }

    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error> {
        let (node_name, node_id, strategy_id, strategy_output_handle, state_machine) = self
            .with_ctx_read(|ctx| {
                let node_name = ctx.node_name().to_string();
                let node_id = ctx.node_id().clone();
                let strategy_id = ctx.strategy_id().clone();
                let strategy_output_handle = ctx.strategy_bound_handle().clone();
                let state_machine = ctx.state_machine().clone();
                (node_name, node_id, strategy_id, strategy_output_handle, state_machine)
            })
            .await;

        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(trans_trigger)?
        };

        // 执行转换后需要执行的动作
        for action in transition_result.actions() {
            let (previous_state, current_state) = {
                let state_machine = state_machine.read().await;
                (state_machine.previous_state().clone(), state_machine.current_state().clone())
            };

            match action {
                IndicatorNodeAction::LogTransition => {
                    tracing::info!("[{node_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }
                IndicatorNodeAction::LogNodeState => {
                    tracing::info!("[{node_name}] current state: {:?}", current_state);
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IndicatorNode,
                        log_message.to_string(),
                        current_state,
                        IndicatorNodeAction::LogNodeState,
                        &strategy_output_handle,
                    )
                    .await;
                }
                IndicatorNodeAction::ListenAndHandleExternalEvents => {
                    tracing::info!("[{node_name}] starting to listen external events");
                    let log_message = ListenExternalEventsMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IndicatorNode,
                        log_message.to_string(),
                        current_state,
                        IndicatorNodeAction::ListenAndHandleExternalEvents,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_engine_event().await;
                }
                IndicatorNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] starting to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IndicatorNode,
                        log_message.to_string(),
                        current_state,
                        IndicatorNodeAction::ListenAndHandleNodeEvents,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_source_node_events().await;
                }
                IndicatorNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] starting to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IndicatorNode,
                        log_message.to_string(),
                        current_state,
                        IndicatorNodeAction::ListenAndHandleStrategyCommand,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_command().await;
                }

                IndicatorNodeAction::InitIndicatorLookback => {
                    tracing::info!("@[{node_name}] starting to init indicator lookback");
                    let result = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move {
                                ctx.init_indicator_lookback().await?;
                                Ok::<(), IndicatorNodeError>(())
                            })
                        })
                        .await;
                    match result {
                        Ok(()) => {
                            let log_message = InitLookbackSuccessMsg::new(node_name.clone());
                            NodeUtils::send_run_state_info(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::IndicatorNode,
                                log_message.to_string(),
                                current_state,
                                IndicatorNodeAction::InitIndicatorLookback,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(e) => {
                            NodeUtils::send_run_state_error(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::IndicatorNode,
                                IndicatorNodeAction::InitIndicatorLookback,
                                &e,
                                &strategy_output_handle,
                            )
                            .await;
                            return Err(e.into());
                        }
                    }
                }

                IndicatorNodeAction::InitMinInterval => {
                    let result = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move {
                                ctx.init_min_interval_from_strategy().await?;
                                Ok::<KlineInterval, IndicatorNodeError>(ctx.min_interval().clone())
                            })
                        })
                        .await;
                    match result {
                        Ok(min_interval) => {
                            let log_message = InitMinIntervalSuccessMsg::new(node_name.clone(), min_interval.to_string());
                            NodeUtils::send_run_state_info(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::IndicatorNode,
                                log_message.to_string(),
                                current_state,
                                IndicatorNodeAction::InitMinInterval,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(e) => {
                            NodeUtils::send_run_state_error(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::IndicatorNode,
                                IndicatorNodeAction::InitMinInterval,
                                &e,
                                &strategy_output_handle,
                            )
                            .await;
                            return Err(e.into());
                        }
                    }
                }

                IndicatorNodeAction::CalculateIndicator => {
                    tracing::info!("[{node_name}] starting to calculate indicator");
                    let log_message = CalculateIndicatorMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IndicatorNode,
                        log_message.to_string(),
                        current_state.clone(),
                        IndicatorNodeAction::CalculateIndicator,
                        &strategy_output_handle,
                    )
                    .await;

                    let cal_result = self
                        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.calculate_indicator().await }))
                        .await;
                    // 计算指标，在作用域内获取和释放写锁

                    // 在释放写锁后发送状态事件
                    match cal_result {
                        Ok(_) => {
                            let success_msg = CalculateIndicatorSuccessMsg::new(node_name.clone());
                            NodeUtils::send_run_state_info(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::IndicatorNode,
                                success_msg.to_string(),
                                current_state,
                                IndicatorNodeAction::CalculateIndicator,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(e) => {
                            NodeUtils::send_run_state_error(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::IndicatorNode,
                                IndicatorNodeAction::CalculateIndicator,
                                &e,
                                &strategy_output_handle,
                            )
                            .await;
                            return Err(e.into());
                        }
                    }
                }
                IndicatorNodeAction::CancelAsyncTask => {
                    tracing::debug!("[{node_name}] cancel async task");
                    self.with_ctx_read(|ctx| {
                        ctx.request_cancel();
                    })
                    .await;
                }
                IndicatorNodeAction::LogError(error) => {
                    tracing::error!("[{node_name}] error occurred: {}", error);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
