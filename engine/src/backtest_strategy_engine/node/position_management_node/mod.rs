pub mod position_management_node_context;
pub mod position_management_node_state_machine;
pub mod position_management_node_types;
// pub mod position_management_node_log_message;

use super::node_message::common_log_message::*;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_state_machine::*;
use crate::backtest_strategy_engine::node::{BacktestNodeTrait, NodeType};
use async_trait::async_trait;
use event_center::communication::backtest_strategy::{NodeCommandReceiver, StrategyCommandSender};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use heartbeat::Heartbeat;
use position_management_node_context::PositionNodeContext;
use position_management_node_state_machine::*;
use position_management_node_types::*;
use sea_orm::DatabaseConnection;
use snafu::ResultExt;
use star_river_core::custom_type::{NodeId, NodeName, PlayIndex, StrategyId};
use star_river_core::error::engine_error::node_error::position_management_node_error::*;
use star_river_core::error::engine_error::strategy_engine_error::node_error::*;
use star_river_core::virtual_trading_system::event::VirtualTradingSystemEventReceiver;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use virtual_trading::VirtualTradingSystem;
use super::node_utils::NodeUtils;

#[derive(Debug, Clone)]
pub struct PositionManagementNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl PositionManagementNode {
    pub fn new(
        node_config: serde_json::Value,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, PositionManagementNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) = Self::check_position_management_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::PositionManagementNode,
            Box::new(PositionNodeStateMachine::new(node_id, node_name)),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        );
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(PositionNodeContext {
                base_context,
                backtest_config,
                database,
                heartbeat,
                virtual_trading_system,
                virtual_trading_system_event_receiver,
            }))),
        })
    }

    fn check_position_management_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, PositionNodeBacktestConfig), PositionManagementNodeError> {
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
            serde_json::from_value::<PositionNodeBacktestConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {})?;
        Ok((strategy_id, node_id, node_name, backtest_config))
    }

    async fn listen_virtual_trading_system_events(&self) -> Result<(), String> {
        let (virtual_trading_system_event_receiver, cancel_token, node_id) = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let position_management_node_context = context_guard.as_any().downcast_ref::<PositionNodeContext>().unwrap();

            let receiver = position_management_node_context.virtual_trading_system_event_receiver.resubscribe();
            let cancel_token = position_management_node_context.get_cancel_token().clone();
            let node_id = position_management_node_context.get_node_id().clone();
            (receiver, cancel_token, node_id)
        };

        // 创建一个流，用于接收节点传递过来的message
        let mut stream = BroadcastStream::new(virtual_trading_system_event_receiver);
        let context = self.get_context();
        // 节点接收数据
        tracing::info!(node_id = %node_id, "开始监听虚拟交易系统事件。");
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 虚拟交易系统事件监听任务已中止", node_id);
                        break;
                    }
                    // 接收消息
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                // tracing::debug!("{} 收到消息: {:?}", node_id, message);
                                let mut context_guard = context.write().await;
                                let position_management_node_context = context_guard.as_any_mut().downcast_mut::<PositionNodeContext>().unwrap();
                                position_management_node_context.handle_virtual_trading_system_event(event).await.unwrap();
                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收消息错误: {}", node_id, e);
                            }
                            None => {
                                tracing::warn!("节点{}所有消息流已关闭", node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl BacktestNodeTrait for PositionManagementNode {
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
        tracing::debug!("[{node_name}] setting strategy output handle: {}", strategy_output_handle_id);
        self.add_output_handle(strategy_output_handle_id, tx).await;

        let position_operations = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let position_management_node_context = context_guard.as_any().downcast_ref::<PositionNodeContext>().unwrap();
            position_management_node_context.backtest_config.position_operations.clone()
        };
        // 为每一个订单添加出口
        for position_operation in position_operations.iter() {
            let success_output_handle_id = format!(
                "{}_{}_success_output_{}",
                node_id,
                position_operation.position_operation.to_string(),
                position_operation.position_operation_id
            );
            let failed_output_handle_id = format!(
                "{}_{}_failed_output_{}",
                node_id,
                position_operation.position_operation.to_string(),
                position_operation.position_operation_id
            );
            let (success_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let (failed_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            tracing::debug!("[{node_name}] setting success output handle: {}", success_output_handle_id);
            self.add_output_handle(success_output_handle_id, success_tx).await;
            tracing::debug!("[{node_name}] setting failed output handle: {}", failed_output_handle_id);
            self.add_output_handle(failed_output_handle_id, failed_tx).await;
        }
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("================={}====================", self.get_node_name().await);
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await?;

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("{:?}: 初始化完成", self.get_state_machine().await.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await?;

        // 等待所有任务结束
        self.cancel_task().await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await?;
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let strategy_id = self.get_strategy_id().await;
        let strategy_output_handle = self.get_strategy_output_handle().await;

        // 获取状态管理器并执行转换
        let mut state_machine = self.get_state_machine().await;
        let transition_result = state_machine.transition(event)?;

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(position_node_state_action) = action.as_any().downcast_ref::<PositionManagementNodeStateAction>() {
                let current_state = state_machine.current_state();
                match position_node_state_action {
                    PositionManagementNodeStateAction::LogTransition => {
                        tracing::info!(
                            "[{node_name}] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    PositionManagementNodeStateAction::LogNodeState => {
                        tracing::info!("[{node_name}] current state: {:?}", current_state);

                        // 发送节点状态日志事件
                        let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), PositionManagementNodeStateAction::LogNodeState.to_string(), &strategy_output_handle).await;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("[{node_name}] starting to listen external events");
                        let log_message = ListenExternalEventsMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), PositionManagementNodeStateAction::ListenAndHandleExternalEvents.to_string(), &strategy_output_handle).await;
                        
                        self.listen_external_events().await;
                    }
                    PositionManagementNodeStateAction::RegisterTask => {
                        tracing::info!("[{node_name}] registering position monitoring task");
                        let log_message = RegisterTaskMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), PositionManagementNodeStateAction::RegisterTask.to_string(), &strategy_output_handle).await;

                        let mut context_guard = self.context.write().await;
                        let _position_node_context = context_guard.as_any_mut().downcast_mut::<PositionNodeContext>().unwrap();
                        // position_node_context.monitor_unfilled_order().await;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}] starting to listen node events");
                        let log_message = ListenNodeEventsMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), PositionManagementNodeStateAction::ListenAndHandleNodeEvents.to_string(), &strategy_output_handle).await;
                        self.listen_node_events().await;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("[{node_name}] starting to listen strategy command");
                        let log_message = ListenStrategyCommandMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), PositionManagementNodeStateAction::ListenAndHandleStrategyCommand.to_string(), &strategy_output_handle).await;
                        self.listen_strategy_command().await;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleVirtualTradingSystemEvent => {
                        tracing::info!("[{node_name}] starting to listen virtual trading system events");
                        let log_message = ListenVirtualTradingSystemEventMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), PositionManagementNodeStateAction::ListenAndHandleVirtualTradingSystemEvent.to_string(), &strategy_output_handle).await;
                        let _ = self.listen_virtual_trading_system_events().await;
                    }
                    PositionManagementNodeStateAction::LogError(error) => {
                        tracing::error!("[{node_name}] error occurred: {}", error);
                    }
                }
            }
        }

        // 所有动作执行完毕后更新节点最新的状态
        self.context.write().await.set_state_machine(state_machine.clone_box());
        Ok(())
    }
}
