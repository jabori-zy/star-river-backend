use super::StartNode;

use async_trait::async_trait;
use strategy_core::event::node_common_event::CommonEvent;
use crate::{
    error::node_error::BacktestNodeError, 
    node_list_new::node_state_machine::NodeStateTransTrigger};
use strategy_core::node::node_trait::{NodeLifecycle, NodeContextAccessor};
use strategy_core::node::context_trait::{NodeTaskControlExt, NodeHandleExt, NodeIdentityExt, NodeStateMachineExt};
use strategy_core::node::node_state_machine::StateMachine;
use super::state_machine::StartNodeAction;
use crate::node_list_new::node_message::common_log_message::*;
use crate::node_list_new::node_message::start_node_log_message::*;
use crate::node_list_new::node_utils::NodeUtils;
use strategy_core::event::log_event::NodeStateLogEvent;
use strategy_core::node::node_trait::NodeEventListener;


#[async_trait]
impl NodeLifecycle for StartNode {

    type Error = BacktestNodeError;

    type Trigger = NodeStateTransTrigger;

    async fn init(&self) -> Result<(), Self::Error> {
        let node_name = self.with_ctx_read(|ctx| {
            ctx.node_name().to_string()
        }).await;
        tracing::info!("=================init node [{node_name}]====================");
        tracing::info!("[{node_name}] start to init");
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

    async fn stop(&self) -> Result<(), BacktestNodeError> {
        let node_name = self.with_ctx_read(|ctx| {
            ctx.node_name().to_string()
        }).await;
        tracing::info!("=================stop node [{node_name}]====================");
        tracing::info!("[{node_name}] start to stop");
        self.update_node_state(NodeStateTransTrigger::StartStop).await?;
        // 切换为stopped状态
        self.update_node_state(NodeStateTransTrigger::FinishStop).await?;
        Ok(())
    }

    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error> {
        let (node_name, node_id, strategy_id, strategy_output_handle, state_machine) = self
            .with_ctx_read(|ctx| {
                let node_name = ctx.node_name().clone();
                let node_id = ctx.node_id().clone();
                let strategy_id = ctx.strategy_id();
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
            // 克隆actions避免移动问题
            
            let current_state = {
                let state_machine = state_machine.read().await;
                state_machine.current_state().clone()
            };
            match action {
                StartNodeAction::LogTransition => {
                    tracing::debug!(
                        "[{node_name}] state transition: {:?} -> {:?}",
                        current_state,
                        transition_result.new_state()
                    );
                }
                StartNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] starting to listen strategy command");
                    let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                    let log_event: CommonEvent = NodeStateLogEvent::success(
                        strategy_id.clone(),
                        node_id.clone(),
                        node_name.clone(),
                        current_state.to_string(),
                        StartNodeAction::ListenAndHandleStrategyCommand.to_string(),
                        log_message.to_string(),
                    ).into();
                    let _ = strategy_output_handle.send(log_event.into());
                    self.listen_node_command().await;
                }
                StartNodeAction::ListenAndHandlePlayIndex => {
                    tracing::info!("[{node_name}] starting to listen play index change");
                    let log_message = ListenPlayIndexChangeMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        StartNodeAction::ListenAndHandlePlayIndex.to_string(),
                        &strategy_output_handle,
                    ).await;
                    self.listen_play_index_change().await;
                }
                StartNodeAction::InitVirtualTradingSystem => {
                    tracing::info!("[{node_name}] start to init virtual trading system");
                    self.with_ctx_read_async(|ctx| {
                        Box::pin(async move {
                            ctx.init_virtual_trading_system().await;
                        })
                    }).await;
                    let log_message = InitVirtualTradingSystemMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        StartNodeAction::InitVirtualTradingSystem.to_string(),
                        &strategy_output_handle,
                    ).await;
                }
                StartNodeAction::InitStrategyStats => {
                    tracing::info!("[{node_name}] start to init strategy stats");
                    
                    self.with_ctx_read_async(|ctx| {
                        Box::pin(async move {
                            ctx.init_strategy_stats().await;
                        })
                    }).await;
                    let log_message = InitStrategyStatsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        StartNodeAction::InitStrategyStats.to_string(),
                        &strategy_output_handle,
                    ).await;
                }
                StartNodeAction::InitCustomVariables => {
                    tracing::info!("[{node_name}] start to init custom variables");
                    self.with_ctx_read_async(|ctx| {
                        Box::pin(async move {
                            ctx.init_custom_variables().await;
                        })
                    }).await;
                    let log_message = InitCustomVariableMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        StartNodeAction::InitCustomVariables.to_string(),
                        &strategy_output_handle,
                    ).await;
                    
                }
                StartNodeAction::LogNodeState => {
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        StartNodeAction::LogNodeState.to_string(),
                        &strategy_output_handle,
                    ).await;
                    
                }
                StartNodeAction::CancelAsyncTask => {
                    tracing::debug!("[{node_name}] cancel async task");
                    self.with_ctx_read(|ctx| {
                        ctx.request_cancel();
                    }).await;
                }
                _ => {}
                
            }
            // 更新状态
            // {
            //     let mut state_guard = self.context.write().await;
            //     state_guard.set_state_machine(state_machine.clone_box());
            // }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }

}

