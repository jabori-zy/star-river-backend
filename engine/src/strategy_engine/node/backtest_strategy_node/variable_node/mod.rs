mod variable_node_context;
mod variable_node_state_machine;

use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use crate::strategy_engine::node::node_state_machine::BacktestNodeStateTransitionEvent;
use crate::strategy_engine::node::{BacktestNodeTrait, NodeType};
use async_trait::async_trait;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use snafu::ResultExt;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use types::node::variable_node::*;
use types::strategy::node_event::BacktestNodeEvent;
use variable_node_context::VariableNodeContext;
use variable_node_state_machine::{VariableNodeStateAction, VariableNodeStateMachine};

use tokio::sync::Mutex;
use event_center::{command::backtest_strategy_command::StrategyCommandReceiver};
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use virtual_trading::VirtualTradingSystem;
use types::custom_type::{NodeId, NodeName, PlayIndex, StrategyId};
use types::error::engine_error::node_error::get_variable_node::ConfigFieldValueNullSnafu;
use types::error::engine_error::strategy_engine_error::node_error::*;
use types::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::get_variable_node::*;
use super::node_message::common_log_message::*;
use types::strategy::node_event::NodeStateLogEvent;

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl VariableNode {
    pub fn new(
        node_config: serde_json::Value,
        // event_publisher: EventPublisher,
        // command_publisher: CommandPublisher,
        // command_receiver: Arc<Mutex<CommandReceiver>>,
        // response_event_receiver: EventReceiver,
        heartbeat: Arc<Mutex<Heartbeat>>,
        database: DatabaseConnection,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, GetVariableNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) =
            Self::check_get_variable_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::GetVariableNode,
            // event_publisher,
            // vec![response_event_receiver],
            // command_publisher,
            // command_receiver,
            Box::new(VariableNodeStateMachine::new(node_id, node_name)),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        );
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(VariableNodeContext {
                base_context,
                backtest_config,
                heartbeat,
                database,
                virtual_trading_system,
            }))),
        })
    }

    fn check_get_variable_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, VariableNodeBacktestConfig), GetVariableNodeError>
    {
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

        let backtest_config =
            serde_json::from_value::<VariableNodeBacktestConfig>(backtest_config_json)
                .context(ConfigDeserializationFailedSnafu {})?;
        Ok((strategy_id, node_id, node_name, backtest_config))
    }
}

#[async_trait]
impl BacktestNodeTrait for VariableNode {
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
        tracing::debug!("{}: 设置节点默认出口", self.get_node_id().await);
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting default output handle");
        self.add_output_handle(default_output_handle_id, tx).await;

        let variable_configs = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let variable_node_context = context_guard
                .as_any()
                .downcast_ref::<VariableNodeContext>()
                .unwrap();
            variable_node_context
                .backtest_config
                .variable_configs
                .clone()
        };

        for variable in variable_configs {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let output_handle_id = format!("{}_output_{}", node_id, variable.config_id);
            self.add_output_handle(output_handle_id, tx).await;
        }

        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!(
            "================={}====================",
            self.get_node_name().await
        );
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize)
            .await?;

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!(
            "{:?}: 初始化完成",
            self.get_state_machine().await.current_state()
        );
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete)
            .await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
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
            if let Some(variable_node_state_action) =
                action.as_any().downcast_ref::<VariableNodeStateAction>()
            {
                let current_state = state_machine.current_state();
                match variable_node_state_action {
                    VariableNodeStateAction::LogTransition => {
                        tracing::info!(
                            "[{node_name}({node_id})] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );

                        // 发送状态转换日志事件
                        let log_message = format!(
                            "状态转换: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                        let log_event = NodeStateLogEvent::success(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            variable_node_state_action.to_string(),
                            log_message,
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }
                    VariableNodeStateAction::LogNodeState => {
                        tracing::info!(
                            "[{node_name}({node_id})] current state: {:?}",
                            current_state
                        );

                        // 发送节点状态日志事件
                        let log_message = NodeStateLogMsg::new(
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                        );
                        let log_event = NodeStateLogEvent::success(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            variable_node_state_action.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }
                    VariableNodeStateAction::RegisterTask => {
                        tracing::info!(
                            "[{node_name}({node_id})] registering variable retrieval task"
                        );
                        let log_message = RegisterTaskMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            variable_node_state_action.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());

                        // 注册任务的具体实现(当前已注释)
                        // let context = self.get_context();
                        // let mut state_guard = context.write().await;
                        // if let Some(get_variable_node_context) = state_guard.as_any_mut().downcast_mut::<GetVariableNodeContext>() {
                        //     let backtest_config = get_variable_node_context.backtest_config.clone();
                        //     let get_variable_type = backtest_config.get_variable_type.clone();
                        //     // 如果获取变量类型为定时触发，则注册任务
                        //     if let GetVariableType::Timer = get_variable_type {
                        //         get_variable_node_context.register_task().await;
                        //     }
                        // }
                    }
                    VariableNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}({node_id})] starting to listen node events");
                        let log_message =
                            ListenNodeEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            variable_node_state_action.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_node_events().await;
                    }
                    VariableNodeStateAction::ListenAndHandleStrategyInnerEvents => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen strategy inner events"
                        );
                        let log_message =
                            ListenStrategyInnerEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            variable_node_state_action.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_inner_events().await;
                    }
                    VariableNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen strategy command"
                        );
                        let log_message =
                            ListenStrategyCommandMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            variable_node_state_action.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_command().await;
                    }
                    VariableNodeStateAction::LogError(error) => {
                        tracing::error!("[{node_name}({node_id})] error occurred: {}", error);
                    }
                    VariableNodeStateAction::CancelAsyncTask => {
                        tracing::debug!("[{node_name}({node_id})] cancel async task");
                        self.cancel_task().await;
                    }
                }
            }
        }

        // 所有动作执行完毕后更新节点最新的状态
        self.context
            .write()
            .await
            .set_state_machine(state_machine.clone_box());
        Ok(())
    }
}
