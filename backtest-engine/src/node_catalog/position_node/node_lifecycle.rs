use async_trait::async_trait;
use strategy_core::node::{
    context_trait::{NodeHandleExt, NodeIdentityExt, NodeStateMachineExt, NodeTaskControlExt},
    node_state_machine::StateMachine,
    node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
};
use tokio::time::Duration;

use super::PositionNode;
use crate::{
    node::{
        node_error::BacktestNodeError,
        node_message::common_log_message::{
            ListenExternalEventsMsg, ListenNodeEventsMsg, ListenStrategyCommandMsg, ListenVirtualTradingSystemEventMsg, NodeStateLogMsg,
            RegisterTaskMsg,
        },
        node_state_machine::NodeStateTransTrigger,
        node_utils::NodeUtils,
    },
    node_catalog::position_node::state_machine::PositionNodeAction,
};

#[async_trait]
impl NodeLifecycle for PositionNode {
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

        // Execute actions after state transition
        for action in transition_result.actions() {
            let (previous_state, current_state) = {
                let state_machine = state_machine.read().await;
                (state_machine.previous_state().clone(), state_machine.current_state().clone())
            };

            match action {
                PositionNodeAction::LogTransition => {
                    tracing::debug!("[{node_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }
                PositionNodeAction::LogNodeState => {
                    tracing::info!("[{node_name}] current state: {:?}", current_state);

                    // Send node state log event
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        PositionNodeAction::LogNodeState.to_string(),
                        &strategy_output_handle,
                    )
                    .await;
                }
                PositionNodeAction::ListenAndHandleExternalEvents => {
                    tracing::info!("[{node_name}] start to listen external events");
                    let log_message = ListenExternalEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        PositionNodeAction::ListenAndHandleExternalEvents.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_engine_event().await;
                }
                PositionNodeAction::RegisterTask => {
                    tracing::info!("[{node_name}] registering position monitoring task");
                    let log_message = RegisterTaskMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        PositionNodeAction::RegisterTask.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    // Register task implementation (if needed)
                    // self.with_ctx_write_async(|ctx| {
                    //     Box::pin(async move {
                    //         ctx.monitor_position().await
                    //     })
                    // }).await;
                }
                PositionNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] start to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        PositionNodeAction::ListenAndHandleNodeEvents.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_source_node_events().await;
                }
                PositionNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] start to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        PositionNodeAction::ListenAndHandleStrategyCommand.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_command().await;
                }
                PositionNodeAction::ListenAndHandleVirtualTradingSystemEvent => {
                    tracing::info!("[{node_name}] start to listen virtual trading system events");
                    let log_message = ListenVirtualTradingSystemEventMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        PositionNodeAction::ListenAndHandleVirtualTradingSystemEvent.to_string(),
                        &strategy_output_handle,
                    )
                    .await;

                    // Listen to virtual trading system events implementation (if needed)
                    // let _ = self.listen_virtual_trading_system_events().await;
                }
                PositionNodeAction::LogError(error) => {
                    tracing::error!("[{node_name}] error occurred: {}", error);
                }
                PositionNodeAction::CancelAsyncTask => {
                    tracing::debug!("[{node_name}] cancel async task");
                    self.with_ctx_read_async(|ctx| {
                        Box::pin(async move {
                            ctx.request_cancel();
                        })
                    })
                    .await;
                }
            }
        }
        Ok(())
    }
}
