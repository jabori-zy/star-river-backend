pub mod condition;
mod if_else_node_context;
mod if_else_node_state_machine;
pub mod if_else_node_type;
mod utils;

use super::if_else_node::if_else_node_state_machine::{
    IfElseNodeStateAction, IfElseNodeStateManager,
};
use super::node_message::common_log_message::*;
use super::node_message::if_else_node_log_message::*;
use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use crate::strategy_engine::node::node_state_machine::*;
use crate::strategy_engine::node::node_types::NodeType;
use crate::strategy_engine::node::BacktestNodeTrait;
use async_trait::async_trait;
use condition::Case;
use event_center::command::backtest_strategy_command::StrategyCommandReceiver;
use if_else_node_context::IfElseNodeContext;
use if_else_node_type::IfElseNodeBacktestConfig;
use snafu::ResultExt;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use types::custom_type::PlayIndex;
use types::custom_type::{NodeId, NodeName, StrategyId};
use types::error::engine_error::strategy_engine_error::node_error::if_else_node_error::*;
use types::error::engine_error::strategy_engine_error::node_error::*;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::node_event::BacktestNodeEvent;
use types::strategy::node_event::NodeStateLogEvent;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;

// 条件分支节点
#[derive(Debug, Clone)]
pub struct IfElseNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl IfElseNode {
    pub fn new(
        node_config: serde_json::Value,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, IfElseNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) =
            Self::check_if_else_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IfElseNode,
            Box::new(IfElseNodeStateManager::new(
                BacktestNodeRunState::Created,
                node_id,
                node_name,
            )),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        );
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(IfElseNodeContext {
                base_context,
                received_flag: HashMap::new(),
                received_message: HashMap::new(),
                backtest_config,
            }))),
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
        let cases = serde_json::from_value::<Vec<Case>>(cases_json)
            .context(ConfigDeserializationFailedSnafu {})?;
        let backtest_config = IfElseNodeBacktestConfig { cases };
        Ok((strategy_id, node_id, node_name, backtest_config))
    }

    async fn evaluate(&self) {
        let (node_id, cancel_token) = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let node_id = context_guard.get_node_id().clone();
            let cancel_token = context_guard.get_cancel_token().clone();
            (node_id, cancel_token)
        };

        let context = self.context.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点条件判断进程已中止", node_id);
                        break;
                    }
                    _ = async {
                        // 使用更短的锁持有时间
                        let should_evaluate = {
                            let context_guard = context.read().await; // 使用读锁检查状态
                            let if_else_node_context = context_guard
                                .as_any()
                                .downcast_ref::<IfElseNodeContext>()
                                .expect("转换为IfElseNodeContext失败");

                            if_else_node_context.is_all_value_received()
                        };

                        if should_evaluate {
                            let mut context_guard = context.write().await; // 只有需要时才获取写锁
                            let if_else_node_context = context_guard
                                .as_any_mut()
                                .downcast_mut::<IfElseNodeContext>()
                                .expect("转换为IfElseNodeContext失败");

                            // 双重检查，防止竞态条件
                            if if_else_node_context.is_all_value_received() {
                                tracing::debug!("{}: 所有值已接收，开始评估", node_id);
                                if_else_node_context.evaluate().await;
                                if_else_node_context.reset_received_flag();
                            }
                        }

                        // 动态调整sleep时间
                        let sleep_duration = if should_evaluate { 10 } else { 50 };
                        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration)).await;
                    } => {}
                }
            }
        });
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

    async fn set_output_handle(&mut self) {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        // let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        // let default_output_handle_id = format!("{}_default_output", node_id);
        // tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting default output handle");
        // self.add_output_handle(default_output_handle_id, tx).await;

        // 添加else出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let else_output_handle_id = format!("{}_else_output", node_id); // else分支作为默认出口
        tracing::debug!(node_id = %node_id, node_name = %node_name, else_output_handle_id = %else_output_handle_id, "setting ELSE output handle");
        self.add_output_handle(else_output_handle_id, tx).await;

        let cases = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let if_else_node_context = context_guard
                .as_any()
                .downcast_ref::<IfElseNodeContext>()
                .unwrap();
            if_else_node_context.backtest_config.cases.clone()
        };

        for case in cases {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let case_output_handle_id = case.output_handle_id.clone();
            self.add_output_handle(case_output_handle_id, tx).await;
        }
        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!(
            "================={}====================",
            self.context.read().await.get_node_name()
        );
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize)
            .await?;

        tracing::info!(
            "{:?}: 初始化完成",
            self.context
                .read()
                .await
                .get_state_machine()
                .current_state()
        );
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete)
            .await?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop)
            .await?;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete)
            .await?;
        Ok(())
    }

    async fn update_node_state(
        &mut self,
        event: BacktestNodeStateTransitionEvent,
    ) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let strategy_id = self.get_strategy_id().await;
        let strategy_output_handle = self.get_strategy_output_handle().await;

        // 获取状态管理器并执行转换
        let mut state_machine = self.get_state_machine().await;
        let transition_result = state_machine.transition(event)?;

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(if_else_node_state_action) =
                action.as_any().downcast_ref::<IfElseNodeStateAction>()
            {
                let current_state = state_machine.current_state();
                match if_else_node_state_action {
                    IfElseNodeStateAction::LogTransition => {
                        tracing::info!(
                            "[{node_name}({node_id})] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    IfElseNodeStateAction::ListenAndHandleStrategySignal => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen strategy signal"
                        );
                        let log_message =
                            ListenStrategySignalMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::ListenAndHandleStrategySignal.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }
                    IfElseNodeStateAction::LogNodeState => {
                        tracing::info!(
                            "[{node_name}({node_id})] current state: {:?}",
                            current_state
                        );
                        let log_message = NodeStateLogMsg::new(
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                        );
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::LogNodeState.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }

                    IfElseNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}({node_id})] starting to listen node events");
                        let log_message =
                            ListenNodeEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::ListenAndHandleNodeEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_node_events().await;
                    }
                    IfElseNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen strategy inner events"
                        );
                        let log_message =
                            ListenStrategyInnerEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::ListenAndHandleInnerEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_inner_events().await;
                    }
                    IfElseNodeStateAction::InitReceivedData => {
                        tracing::info!("[{node_name}({node_id})] initializing received data flags");
                        let log_message =
                            InitReceivedDataMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::InitReceivedData.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(if_else_node_context) =
                            state_guard.as_any_mut().downcast_mut::<IfElseNodeContext>()
                        {
                            if_else_node_context.init_received_data().await;
                        }
                    }
                    IfElseNodeStateAction::Evaluate => {
                        tracing::info!("[{node_name}({node_id})] starting condition evaluation");
                        let log_message =
                            StartConditionEvaluationMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::Evaluate.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.evaluate().await;
                    }
                    IfElseNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen strategy command"
                        );
                        let log_message =
                            ListenStrategyCommandMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IfElseNodeStateAction::ListenAndHandleStrategyCommand.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_command().await;
                    }

                    IfElseNodeStateAction::CancelAsyncTask => {
                        tracing::debug!("[{node_name}({node_id})] cancel async task");
                        self.cancel_task().await;
                    }
                    _ => {}
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    self.context
                        .write()
                        .await
                        .set_state_machine(state_machine.clone_box());
                }
            }
        }

        Ok(())
    }
}

// 比较操作符
