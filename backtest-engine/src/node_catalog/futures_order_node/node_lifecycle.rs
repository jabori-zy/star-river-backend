use async_trait::async_trait;
use strategy_core::{
    NodeType,
    node::{
        context_trait::{NodeHandleExt, NodeInfoExt, NodeStateMachineExt, NodeTaskControlExt},
        node_state_machine::StateMachine,
        node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
    },
};

use super::FuturesOrderNode;
use crate::{
    node::{
        node_error::BacktestNodeError,
        node_message::{
            common_log_message::{
                ListenNodeEventsMsg, ListenStrategyCommandMsg, ListenVirtualTradingSystemEventMsg, NodeStateLogMsg,
            },
            futures_order_node_log_message::{GetSymbolInfoMsg, GetSymbolInfoSuccessMsg},
        },
        node_state_machine::NodeStateTransTrigger,
        node_utils::NodeUtils,
    },
    node_catalog::futures_order_node::state_machine::FuturesOrderNodeAction,
};

#[async_trait]
impl NodeLifecycle for FuturesOrderNode {
    type Error = BacktestNodeError;
    type Trigger = NodeStateTransTrigger;
    async fn init(&self) -> Result<(), Self::Error> {
        NodeUtils::init_node(self, Some(500)).await
    }
    async fn stop(&self) -> Result<(), Self::Error> {
        NodeUtils::stop_node(self, Some(1000)).await
    }
    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error> {
        let (strategy_id, node_id, node_name, strategy_output_handle, state_machine) = self
            .with_ctx_read(|ctx| {
                let strategy_id = ctx.strategy_id().clone();
                let node_id = ctx.node_id().clone();
                let node_name = ctx.node_name().clone();
                let strategy_output_handle = ctx.strategy_bound_handle().clone();
                let state_machine = ctx.state_machine().clone();
                (strategy_id, node_id, node_name, strategy_output_handle, state_machine)
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
                FuturesOrderNodeAction::LogTransition => {
                    tracing::debug!("[{node_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }
                FuturesOrderNodeAction::LogNodeState => {
                    tracing::info!("[{node_name}] current state: {:?}", current_state);

                    // 发送节点状态日志事件
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::FuturesOrderNode,
                        log_message.to_string(),
                        current_state,
                        FuturesOrderNodeAction::LogNodeState,
                        &strategy_output_handle,
                    )
                    .await;
                }
                FuturesOrderNodeAction::GetSymbolInfo => {
                    tracing::info!("[{node_name}] start to get symbol info");
                    let log_message = GetSymbolInfoMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::FuturesOrderNode,
                        log_message.to_string(),
                        current_state.clone(),
                        FuturesOrderNodeAction::GetSymbolInfo,
                        &strategy_output_handle,
                    )
                    .await;
                    let result = self
                        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.get_symbol_info().await }))
                        .await;
                    match result {
                        Ok(_) => {
                            let log_message = GetSymbolInfoSuccessMsg::new(node_name.clone());
                            NodeUtils::send_run_state_info(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::FuturesOrderNode,
                                log_message.to_string(),
                                current_state,
                                FuturesOrderNodeAction::GetSymbolInfo,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(err) => {
                            NodeUtils::send_run_state_error(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::FuturesOrderNode,
                                FuturesOrderNodeAction::GetSymbolInfo,
                                &err,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                    }
                }
                // FuturesOrderNodeAction::RegisterTask => {
                //     tracing::info!("[{node_name}] registering heartbeat monitoring task");
                //     let log_message = RegisterTaskMsg::new(node_name.clone());
                //     NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state, FuturesOrderNodeAction::RegisterTask.to_string(), &strategy_output_handle).await;

                //     self.with_ctx_write_async(|ctx| {
                //         Box::pin(async move {
                //             ctx.monitor_unfilled_order().await
                //         })
                //     }).await;
                // }
                FuturesOrderNodeAction::ListenAndHandleNodeEvents => {
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::FuturesOrderNode,
                        log_message.to_string(),
                        current_state,
                        FuturesOrderNodeAction::ListenAndHandleNodeEvents,
                        &strategy_output_handle,
                    )
                    .await;
                    self.listen_source_node_events_for_independent_order().await;
                }
                FuturesOrderNodeAction::ListenAndHandleCommand => {
                    tracing::info!("[{node_name}] start to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::FuturesOrderNode,
                        log_message.to_string(),
                        current_state,
                        FuturesOrderNodeAction::ListenAndHandleCommand,
                        &strategy_output_handle,
                    )
                    .await;
                    self.listen_command().await;
                }

                FuturesOrderNodeAction::ListenAndHandleVtsEvent => {
                    tracing::info!("[{node_name}] start to listen virtual trading system events");
                    let log_message = ListenVirtualTradingSystemEventMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::FuturesOrderNode,
                        log_message.to_string(),
                        current_state,
                        FuturesOrderNodeAction::ListenAndHandleVtsEvent,
                        &strategy_output_handle,
                    )
                    .await;
                    let _ = self.listen_vts_events().await;
                }
                FuturesOrderNodeAction::LogError(error) => {
                    tracing::error!("[{node_name}] error occurred: {}", error);
                }
                FuturesOrderNodeAction::CancelAsyncTask => {
                    tracing::debug!("[{node_name}] cancel async task");
                    self.with_ctx_read_async(|ctx| {
                        Box::pin(async move {
                            ctx.request_cancel();
                        })
                    })
                    .await;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
