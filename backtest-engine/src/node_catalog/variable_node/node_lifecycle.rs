use async_trait::async_trait;
use strategy_core::{
    NodeType,
    node::{
        context_trait::{NodeHandleExt, NodeInfoExt, NodeStateMachineExt, NodeTaskControlExt},
        node_state_machine::StateMachine,
        node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
    },
};

use super::VariableNode;
use crate::{
    node::{
        node_error::VariableNodeError,
        node_message::{
            common_log_message::{ListenNodeEventsMsg, ListenStrategyCommandMsg, NodeStateLogMsg},
            variable_node_log_message::RegisterVariableRetrievalTaskMsg,
        },
        node_state_machine::NodeStateTransTrigger,
        node_utils::NodeUtils,
    },
    node_catalog::variable_node::state_machine::VariableNodeAction,
};

#[async_trait]
impl NodeLifecycle for VariableNode {
    type Error = VariableNodeError;
    type Trigger = NodeStateTransTrigger;

    async fn init(&self) -> Result<(), Self::Error> {
        NodeUtils::init_node(self, Some(500)).await
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        NodeUtils::stop_node(self, Some(500)).await
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
                VariableNodeAction::LogTransition => {
                    tracing::debug!("[{node_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }
                VariableNodeAction::LogNodeState => {
                    tracing::info!("[{node_name}] current state: {:?}", current_state);

                    // Send node state log event
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::VariableNode,
                        log_message.to_string(),
                        current_state,
                        VariableNodeAction::LogNodeState,
                        &strategy_output_handle,
                    )
                    .await;
                }
                VariableNodeAction::RegisterTask => {
                    tracing::info!("[{node_name}] registering variable retrieval task");
                    let log_message = RegisterVariableRetrievalTaskMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::VariableNode,
                        log_message.to_string(),
                        current_state,
                        VariableNodeAction::RegisterTask,
                        &strategy_output_handle,
                    )
                    .await;

                    // Register task implementation (if needed)
                    // self.with_ctx_write_async(|ctx| {
                    //     Box::pin(async move {
                    //         ctx.register_task().await
                    //     })
                    // }).await;
                }
                VariableNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] start to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::VariableNode,
                        log_message.to_string(),
                        current_state,
                        VariableNodeAction::ListenAndHandleNodeEvents,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_source_node_events().await;
                }
                VariableNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] start to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::VariableNode,
                        log_message.to_string(),
                        current_state,
                        VariableNodeAction::ListenAndHandleStrategyCommand,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_command().await;
                }
                VariableNodeAction::LogError(error) => {
                    tracing::error!("[{node_name}] error occurred: {}", error);
                }
                VariableNodeAction::CancelAsyncTask => {
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
