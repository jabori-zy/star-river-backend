use async_trait::async_trait;
use strategy_core::{
    NodeType,
    node::{
        context_trait::{NodeHandleExt, NodeInfoExt, NodeStateMachineExt, NodeTaskControlExt},
        node_state_machine::StateMachine,
        node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
    },
};

use super::IfElseNode;
use crate::{
    node::{
        node_error::IfElseNodeError,
        node_message::{
            common_log_message::{ListenNodeEventsMsg, ListenStrategyCommandMsg, NodeStateLogMsg},
            if_else_node_log_message::{InitReceivedDataMsg, ListenStrategySignalMsg, StartConditionEvaluationMsg},
        },
        node_state_machine::NodeStateTransTrigger,
        node_utils::NodeUtils,
    },
    node_catalog::if_else_node::state_machine::IfElseNodeAction,
};

#[async_trait]
impl NodeLifecycle for IfElseNode {
    type Error = IfElseNodeError;

    type Trigger = NodeStateTransTrigger;

    async fn init(&self) -> Result<(), Self::Error> {
        NodeUtils::init_node(self, None).await
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        NodeUtils::stop_node(self, Some(1000)).await
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

        // Execute actions after state transition
        for action in transition_result.actions() {
            let (previous_state, current_state) = {
                let state_machine = state_machine.read().await;
                (state_machine.previous_state().clone(), state_machine.current_state().clone())
            };
            match action {
                IfElseNodeAction::LogTransition => {
                    tracing::info!("[{node_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }
                IfElseNodeAction::ListenAndHandleStrategySignal => {
                    tracing::info!("[{node_name}] starting to listen strategy signal");
                    let log_message = ListenStrategySignalMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IfElseNode,
                        log_message.to_string(),
                        current_state,
                        IfElseNodeAction::ListenAndHandleStrategySignal,
                        &strategy_output_handle,
                    )
                    .await;
                }
                IfElseNodeAction::LogNodeState => {
                    tracing::info!("[{node_name}] current state: {:?}", current_state);
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IfElseNode,
                        log_message.to_string(),
                        current_state,
                        IfElseNodeAction::LogNodeState,
                        &strategy_output_handle,
                    )
                    .await;
                }

                IfElseNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] starting to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IfElseNode,
                        log_message.to_string(),
                        current_state,
                        IfElseNodeAction::ListenAndHandleNodeEvents,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_source_node_events().await;
                }
                IfElseNodeAction::InitReceivedData => {
                    tracing::info!("[{node_name}] initializing received data flags");
                    let log_message = InitReceivedDataMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IfElseNode,
                        log_message.to_string(),
                        current_state,
                        IfElseNodeAction::InitReceivedData,
                        &strategy_output_handle,
                    )
                    .await;

                    self.with_ctx_write_async(|ctx| Box::pin(async move { ctx.init_received_data().await }))
                        .await;
                }
                IfElseNodeAction::Evaluate => {
                    tracing::info!("[{node_name}] starting condition evaluation");
                    let log_message = StartConditionEvaluationMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IfElseNode,
                        log_message.to_string(),
                        current_state,
                        IfElseNodeAction::Evaluate,
                        &strategy_output_handle,
                    )
                    .await;
                    self.evaluate().await?;
                }
                IfElseNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] starting to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_run_state_info(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::IfElseNode,
                        log_message.to_string(),
                        current_state,
                        IfElseNodeAction::ListenAndHandleStrategyCommand,
                        &strategy_output_handle,
                    )
                    .await;
                    self.listen_command().await;
                }

                IfElseNodeAction::CancelAsyncTask => {
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
