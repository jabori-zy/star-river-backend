use async_trait::async_trait;
use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeIdentityExt, NodeStateMachineExt, NodeTaskControlExt},
    node_state_machine::StateMachine,
    node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
};
use tokio::time::Duration;

use super::IndicatorNode;
use crate::{
    node::{
        node_error::BacktestNodeError,
        node_message::{
            common_log_message::{
                GetMinIntervalSymbolsSuccessMsg, ListenExternalEventsMsg, ListenNodeEventsMsg, ListenStrategyCommandMsg, NodeStateLogMsg,
            },
            indicator_node_log_message::{CalculateIndicatorMsg, CalculateIndicatorSuccessMsg},
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
        let (node_name, node_id, strategy_id, strategy_output_handle, mut state_machine) = self
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
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        IndicatorNodeAction::LogNodeState.to_string(),
                        &strategy_output_handle,
                    )
                    .await;
                }
                IndicatorNodeAction::ListenAndHandleExternalEvents => {
                    tracing::info!("[{node_name}] starting to listen external events");
                    let log_message = ListenExternalEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        IndicatorNodeAction::ListenAndHandleExternalEvents.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_engine_event().await;
                }
                IndicatorNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] starting to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        IndicatorNodeAction::ListenAndHandleNodeEvents.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_source_node_events().await;
                }
                IndicatorNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] starting to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        IndicatorNodeAction::ListenAndHandleStrategyCommand.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_node_command().await;
                }

                IndicatorNodeAction::InitIndicatorLookback => {
                    tracing::info!("@[{node_name}] starting to init indicator lookback");
                    self.with_ctx_write_async(|ctx| {
                        Box::pin(async move {
                            ctx.init_indicator_lookback().await;
                        })
                    })
                    .await;
                    tracing::info!("[{node_name})] init indicator lookback complete");
                }

                IndicatorNodeAction::GetMinIntervalSymbols => {
                    let _ = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move {
                                ctx.get_min_interval_symbols_from_strategy().await.map(|min_interval_symbols| {
                                    ctx.set_min_interval_symbols(min_interval_symbols);
                                })
                            })
                        })
                        .await;

                    let log_message = GetMinIntervalSymbolsSuccessMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        IndicatorNodeAction::GetMinIntervalSymbols.to_string(),
                        &strategy_output_handle,
                    )
                    .await;
                }

                IndicatorNodeAction::CalculateIndicator => {
                    tracing::info!("[{node_name}] starting to calculate indicator");
                    let log_message = CalculateIndicatorMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        IndicatorNodeAction::CalculateIndicator.to_string(),
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
                            NodeUtils::send_success_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                success_msg.to_string(),
                                current_state.to_string(),
                                IndicatorNodeAction::CalculateIndicator.to_string(),
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(e) => {
                            NodeUtils::send_error_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                IndicatorNodeAction::CalculateIndicator.to_string(),
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
                _ => {}
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
