// pub mod condition;
pub mod if_else_node_context;
mod if_else_node_state_machine;
// pub mod if_else_node_type;
mod utils;

use super::if_else_node::if_else_node_state_machine::{IfElseNodeStateAction, IfElseNodeStateManager};
use super::node_message::common_log_message::*;
use super::node_message::if_else_node_log_message::*;
use crate::backtest_strategy_engine::node::BacktestNodeTrait;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_state_machine::*;
use crate::backtest_strategy_engine::node::node_handles::NodeType;
use async_trait::async_trait;
use star_river_core::node::if_else_node::{Case, IfElseNodeBacktestConfig};
use event_center::communication::backtest_strategy::{
    NodeCommandReceiver, StrategyCommandSender,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use if_else_node_context::IfElseNodeContext;
use snafu::ResultExt;
use star_river_core::custom_type::PlayIndex;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use star_river_core::error::engine_error::strategy_engine_error::node_error::if_else_node_error::*;
use star_river_core::error::engine_error::strategy_engine_error::node_error::*;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use super::node_utils::NodeUtils;
use star_river_core::strategy::node_benchmark::CycleTracker;
use super::context_accessor::BacktestNodeContextAccessor;

// 条件分支节点
#[derive(Debug, Clone)]
pub struct IfElseNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl IfElseNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, IfElseNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_if_else_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IfElseNode,
            Box::new(IfElseNodeStateManager::new(BacktestNodeRunState::Created, node_id, node_name)),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        );
        let if_else_node_context = IfElseNodeContext::new(base_context, node_config);
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(if_else_node_context))),
        })
    }

    fn check_if_else_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, IfElseNodeBacktestConfig), IfElseNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "id".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_data = node_config
            .get("data")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "data".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "nodeName".to_string(),
                }
                .build()
            })?
            .to_owned();
        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "strategyId".to_string(),
                }
                .build()
            })?
            .to_owned() as StrategyId;

        let backtest_config_json = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let cases_json = backtest_config_json
            .get("cases")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "cases".to_string(),
                }
                .build()
            })?
            .to_owned();
        let cases = serde_json::from_value::<Vec<Case>>(cases_json).context(ConfigDeserializationFailedSnafu {})?;
        let backtest_config = IfElseNodeBacktestConfig { cases };
        Ok((strategy_id, node_id, node_name, backtest_config))
    }

    async fn evaluate(&self) -> Result<(), BacktestStrategyNodeError> {
        let (node_id, cancel_token) = self.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
            let node_id = ctx.get_node_id().clone();
            let cancel_token = ctx.get_cancel_token().clone();
            (node_id, cancel_token)
        }).await?;

        let node = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点条件判断进程已中止", node_id);
                        break;
                    }
                    _ = async {
                        // 使用更短的锁持有时间
                        let should_evaluate = match node.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
                                ctx.is_all_value_received()
                            }).await {
                                Ok(result) => result,
                                Err(err) => {
                                    tracing::warn!("[{}] Failed to check if all values received: {}", node_id, err);
                                    false
                                }
                            };
                        if should_evaluate {
                            match node.with_ctx_write_async::<IfElseNodeContext, _>(|ctx| {
                                Box::pin(async move {
                                    let result = ctx.evaluate().await;
                                    ctx.reset_received_flag();
                                    result
                                })
                            }).await {
                                Ok(Ok(())) => {
                                    // 评估成功
                                }
                                Ok(Err(err)) => {
                                    tracing::error!("[{}] Evaluation failed: {:?}", node_id, err);
                                }
                                Err(err) => {
                                    tracing::error!("[{}] Failed to downcast context during evaluation: {}", node_id, err);
                                }
                            }
                        }

                        // 动态调整sleep时间
                        let sleep_duration = 10;
                        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration)).await;
                    } => {}
                }
            }
        });
        Ok(())
    }
}

#[async_trait]
impl BacktestNodeTrait for IfElseNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    async fn set_output_handle(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let (node_id, node_name, cases) = self.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
            let node_id = ctx.get_node_id().clone();
            let node_name = ctx.get_node_name().clone();
            let cases = ctx.node_config.cases.clone();
            (node_id, node_name, cases)
        }).await?;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!("[{node_name}] setting strategy output handle: {}", strategy_output_handle_id);
        self.with_ctx_write::<IfElseNodeContext, _>(|ctx| {
            ctx.add_output_handle(strategy_output_handle_id, tx)
        }).await?;

        // 添加默认出口
        // let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        // let default_output_handle_id = format!("{}_default_output", node_id);
        // tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting default output handle");
        // self.with_ctx_write::<IfElseNodeContext, _>(|ctx| {
        //     ctx.add_output_handle(default_output_handle_id, tx)
        // }).await?;

        // 添加else出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let else_output_handle_id = format!("{}_else_output", node_id); // else分支作为默认出口
        tracing::debug!(
            "[{node_name}] setting ELSE output handle: {}, as default output handle",
            else_output_handle_id
        );
        self.with_ctx_write::<IfElseNodeContext, _>(|ctx| {
            ctx.add_output_handle(else_output_handle_id, tx)
        }).await?;

        for case in cases {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let case_output_handle_id = case.output_handle_id.clone();
            tracing::debug!("[{node_name}] setting case output handle: {}", case_output_handle_id);
            self.with_ctx_write::<IfElseNodeContext, _>(|ctx| {
                ctx.add_output_handle(case_output_handle_id, tx)
            }).await?;
        }
        Ok(())
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let node_name = self.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
            ctx.get_node_name().clone()
        }).await?;
        tracing::info!("================={}====================", node_name);
        tracing::info!("[{node_name}] start init");
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await?;

        let current_state = self.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
            ctx.get_state_machine().current_state()
        }).await?;

        tracing::info!("[{node_name}] init complete: {:?}", current_state);
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await?;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await?;
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError> {
        let (node_name, node_id, strategy_id, strategy_output_handle, mut state_machine) = self.with_ctx_read::<IfElseNodeContext, _>(|ctx| {
            let node_name = ctx.get_node_name().clone();
            let node_id = ctx.get_node_id().clone();
            let strategy_id = ctx.get_strategy_id().clone();
            let strategy_output_handle = ctx.get_strategy_output_handle().clone();
            let state_machine = ctx.get_state_machine();
            (node_name, node_id, strategy_id, strategy_output_handle, state_machine)
        }).await?;

        let transition_result = state_machine.transition(event)?;

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(if_else_node_state_action) = action.as_any().downcast_ref::<IfElseNodeStateAction>() {
                let current_state = state_machine.current_state();
                match if_else_node_state_action {
                    IfElseNodeStateAction::LogTransition => {
                        tracing::info!(
                            "[{node_name}] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    IfElseNodeStateAction::ListenAndHandleStrategySignal => {
                        tracing::info!("[{node_name}] starting to listen strategy signal");
                        let log_message = ListenStrategySignalMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(
                            strategy_id, 
                            node_id.clone(), 
                            node_name.clone(), 
                            log_message.to_string(), 
                            current_state.to_string(), 
                            IfElseNodeStateAction::ListenAndHandleStrategySignal.to_string(), 
                            &strategy_output_handle,
                        ).await;
                        
                    }
                    IfElseNodeStateAction::LogNodeState => {
                        tracing::info!("[{node_name}] current state: {:?}", current_state);
                        let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                        NodeUtils::send_success_status_event(
                            strategy_id, 
                            node_id.clone(), 
                            node_name.clone(), 
                            log_message.to_string(), 
                            current_state.to_string(), 
                            IfElseNodeStateAction::LogNodeState.to_string(), 
                            &strategy_output_handle,
                        ).await;
                        
                    }

                    IfElseNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}] starting to listen node events");
                        let log_message = ListenNodeEventsMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(
                            strategy_id, 
                            node_id.clone(), 
                            node_name.clone(), 
                            log_message.to_string(), 
                            current_state.to_string(), 
                            IfElseNodeStateAction::ListenAndHandleNodeEvents.to_string(), 
                            &strategy_output_handle,
                        ).await;
                        
                        self.listen_node_events().await;
                    }
                    IfElseNodeStateAction::InitReceivedData => {
                        tracing::info!("[{node_name}] initializing received data flags");
                        let log_message = InitReceivedDataMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(
                            strategy_id, 
                            node_id.clone(), 
                            node_name.clone(), 
                            log_message.to_string(), 
                            current_state.to_string(), 
                            IfElseNodeStateAction::InitReceivedData.to_string(), 
                            &strategy_output_handle,
                        ).await;

                        self.with_ctx_write_async::<IfElseNodeContext, _>(|ctx| {
                            Box::pin(async move {
                                ctx.init_received_data().await
                            })
                        }).await?;
                    }
                    IfElseNodeStateAction::Evaluate => {
                        tracing::info!("[{node_name}] starting condition evaluation");
                        let log_message = StartConditionEvaluationMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(
                            strategy_id, 
                            node_id.clone(), 
                            node_name.clone(), 
                            log_message.to_string(), 
                            current_state.to_string(), 
                            IfElseNodeStateAction::Evaluate.to_string(), 
                            &strategy_output_handle,
                        ).await;
                        self.evaluate().await;
                    }
                    IfElseNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("[{node_name}] starting to listen strategy command");
                        let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(
                            strategy_id, 
                            node_id.clone(), 
                            node_name.clone(), 
                            log_message.to_string(), 
                            current_state.to_string(), 
                            IfElseNodeStateAction::ListenAndHandleStrategyCommand.to_string(), 
                            &strategy_output_handle,
                        ).await;
                        self.listen_strategy_command().await;
                    }

                    IfElseNodeStateAction::CancelAsyncTask => {
                        tracing::debug!("[{node_name}] cancel async task");
                        self.cancel_task().await;
                    }
                    _ => {}
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    self.context.write().await.set_state_machine(state_machine.clone_box());
                }
            }
        }

        Ok(())
    }
}

// 比较操作符
