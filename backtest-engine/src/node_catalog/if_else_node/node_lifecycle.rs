use std::time::Duration;

use strategy_core::node::node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle};
use strategy_core::node::context_trait::{NodeStateMachineExt, NodeIdentityExt, NodeTaskControlExt};
use crate::node::node_error::BacktestNodeError;
use crate::node::node_state_machine::NodeStateTransTrigger;
use crate::node_catalog::if_else_node::state_machine::IfElseNodeAction;
use super::IfElseNode;
use async_trait::async_trait;
use strategy_core::node::context_trait::NodeHandleExt;
use strategy_core::node::node_state_machine::StateMachine;
use crate::node::node_message::if_else_node_log_message::ListenStrategySignalMsg;
use crate::node::node_utils::NodeUtils;
use crate::node::node_message::common_log_message::NodeStateLogMsg;
use crate::node::node_message::common_log_message::ListenNodeEventsMsg;
use crate::node::node_message::if_else_node_log_message::StartConditionEvaluationMsg;
use crate::node::node_message::common_log_message::ListenStrategyCommandMsg;
use crate::node::node_message::if_else_node_log_message::InitReceivedDataMsg;


#[async_trait]
impl NodeLifecycle for IfElseNode {
    type Error = BacktestNodeError;

    type Trigger = NodeStateTransTrigger;

    async fn init(&self) -> Result<(), Self::Error> {
        let node_name = self.with_ctx_read(|ctx| {
            ctx.node_name().to_string()
        }).await;
        tracing::info!("================={}====================", node_name);
        tracing::info!("[{node_name}] start init");
        // 开始初始化 created -> Initialize
        self.update_node_state(NodeStateTransTrigger::StartInit).await?;

        let current_state = self.with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.run_state().await.clone()
            })
        }).await;

        tracing::info!("[{node_name}] init complete: {:?}", current_state);
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(NodeStateTransTrigger::FinishInit).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        let node_name = self.with_ctx_read(|ctx| {
            ctx.node_name().to_string()
        }).await;
        tracing::info!("=================stop node [{node_name}]====================");
        tracing::info!("[{node_name}] start to stop");
        self.update_node_state(NodeStateTransTrigger::StartStop).await?;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(NodeStateTransTrigger::FinishStop).await?;
        Ok(())
    }

    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error> {
        let (node_name, node_id, strategy_id, strategy_output_handle, mut state_machine) = self.with_ctx_read(|ctx| {
            let node_name = ctx.node_name().to_string();
            let node_id = ctx.node_id().clone();
            let strategy_id = ctx.strategy_id().clone();
            let strategy_output_handle = ctx.strategy_bound_handle().clone();
            let state_machine = ctx.state_machine().clone();
            (node_name, node_id, strategy_id, strategy_output_handle, state_machine)
        }).await;

        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(trans_trigger)?
        };

        // 执行转换后需要执行的动作
        for action in transition_result.actions() {
            let current_state = {
                let state_machine = state_machine.read().await;
                state_machine.current_state().clone()
            };
            match action {
                IfElseNodeAction::LogTransition => {
                    tracing::info!(
                        "[{node_name}] state transition: {:?} -> {:?}",
                        current_state,
                        transition_result.new_state()
                    );
                }
                IfElseNodeAction::ListenAndHandleStrategySignal => {
                    tracing::info!("[{node_name}] starting to listen strategy signal");
                    let log_message = ListenStrategySignalMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id, 
                        node_id.clone(), 
                        node_name.clone(), 
                        log_message.to_string(), 
                        current_state.to_string(), 
                        IfElseNodeAction::ListenAndHandleStrategySignal.to_string(), 
                        &strategy_output_handle,
                    ).await;
                    
                }
                IfElseNodeAction::LogNodeState => {
                    tracing::info!("[{node_name}] current state: {:?}", current_state);
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_success_status_event(
                        strategy_id, 
                        node_id.clone(), 
                        node_name.clone(), 
                        log_message.to_string(), 
                        current_state.to_string(), 
                        IfElseNodeAction::LogNodeState.to_string(), 
                        &strategy_output_handle,
                    ).await;
                    
                }

                IfElseNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] starting to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id, 
                        node_id.clone(), 
                        node_name.clone(), 
                        log_message.to_string(), 
                        current_state.to_string(), 
                        IfElseNodeAction::ListenAndHandleNodeEvents.to_string(), 
                        &strategy_output_handle,
                    ).await;
                    
                    self.listen_source_node_events().await;
                }
                IfElseNodeAction::InitReceivedData => {
                    tracing::info!("[{node_name}] initializing received data flags");
                    let log_message = InitReceivedDataMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id, 
                        node_id.clone(), 
                        node_name.clone(), 
                        log_message.to_string(), 
                        current_state.to_string(), 
                        IfElseNodeAction::InitReceivedData.to_string(), 
                        &strategy_output_handle,
                    ).await;

                    self.with_ctx_write_async(|ctx| {
                        Box::pin(async move {
                                ctx.init_received_data().await
                            })
                        }).await;
                }
                IfElseNodeAction::Evaluate => {
                    tracing::info!("[{node_name}] starting condition evaluation");
                    let log_message = StartConditionEvaluationMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id, 
                        node_id.clone(), 
                        node_name.clone(), 
                        log_message.to_string(), 
                        current_state.to_string(), 
                        IfElseNodeAction::Evaluate.to_string(), 
                        &strategy_output_handle,
                    ).await;
                    self.evaluate().await;
                }
                IfElseNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] starting to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id, 
                        node_id.clone(), 
                        node_name.clone(), 
                        log_message.to_string(), 
                        current_state.to_string(), 
                        IfElseNodeAction::ListenAndHandleStrategyCommand.to_string(), 
                        &strategy_output_handle,
                    ).await;
                    self.listen_node_command().await;
                }

                IfElseNodeAction::CancelAsyncTask => {
                    tracing::debug!("[{node_name}] cancel async task");
                    self.with_ctx_read_async(|ctx| {
                        Box::pin(async move {
                            ctx.request_cancel();
                        })
                    }).await;
                }
                _ => {}
                }
            
        }

        Ok(())
    }
}