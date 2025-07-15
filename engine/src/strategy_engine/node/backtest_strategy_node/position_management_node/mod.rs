mod position_management_node_context;
pub mod position_management_node_types;
mod position_management_node_state_machine;

use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use std::sync::Arc;
use tokio::sync::RwLock;
use types::strategy::TradeMode;
use position_management_node_types::*;
use event_center::EventPublisher;
use event_center::Event;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use position_management_node_state_machine::*;
use position_management_node_context::PositionNodeContext;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use std::any::Any;
use async_trait::async_trait;
use std::time::Duration;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver, command::backtest_strategy_command::StrategyCommandReceiver};
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use virtual_trading::VirtualTradingSystem;
use types::strategy::node_command::NodeCommandSender;
use crate::strategy_engine::node::node_state_machine::*;
use types::strategy::node_event::BacktestNodeEvent;
use types::virtual_trading_system::event::VirtualTradingSystemEventReceiver;

#[derive(Debug, Clone)]
pub struct PositionManagementNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}


impl PositionManagementNode {
    pub fn new(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        backtest_config: PositionNodeBacktestConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver,
    ) -> Self {
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::PositionManagementNode,
            event_publisher,
            vec![response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(PositionNodeStateMachine::new(node_id, node_name)),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(PositionNodeContext {
                base_context,
                backtest_config,
                database,
                heartbeat,
                virtual_trading_system,
                virtual_trading_system_event_receiver,
            }))),
        }
        
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
        tracing::debug!("{}: 设置节点默认出口", self.get_node_id().await);
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        self.add_output_handle(strategy_output_handle_id, tx).await;

        let position_operations = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let position_management_node_context = context_guard.as_any().downcast_ref::<PositionNodeContext>().unwrap();
            position_management_node_context.backtest_config.position_operations.clone()
        };
        // 为每一个订单添加出口
        for position_operation in position_operations.iter() {
            let success_output_handle_id = format!("{}_{}_success_output{}", node_id,position_operation.position_operation.to_string(), position_operation.position_operation_id);
            let failed_output_handle_id = format!("{}_{}_failed_output{}", node_id,position_operation.position_operation.to_string(), position_operation.position_operation_id);
            let (success_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let (failed_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(success_output_handle_id, success_tx).await;
            self.add_output_handle(failed_output_handle_id, failed_tx).await;
        }

        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.get_node_name().await);
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await.unwrap();

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("{:?}: 初始化完成", self.get_state_machine().await.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await.unwrap();

        // 等待所有任务结束
        self.cancel_task().await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), String> {
        let node_id = self.get_node_id().await;

        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.get_state_machine().await;
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };


        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(position_node_state_action) = action.as_any().downcast_ref::<PositionManagementNodeStateAction>() {
                match position_node_state_action {
                    PositionManagementNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    PositionManagementNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    PositionManagementNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    PositionManagementNodeStateAction::RegisterTask => {
                        tracing::info!("{}: 开始注册心跳任务", node_id);
                        let mut context_guard = self.context.write().await;
                        let position_node_context = context_guard.as_any_mut().downcast_mut::<PositionNodeContext>().unwrap();
                        // position_node_context.monitor_unfilled_order().await;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_node_events().await?;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!("{}: 开始监听策略内部事件", node_id);
                        self.listen_strategy_inner_events().await?;
                    }
                    PositionManagementNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("{}: 开始监听策略命令", node_id);
                        self.listen_strategy_command().await?;
                    }
                    PositionManagementNodeStateAction::LogError(error) => {
                        tracing::error!("{}: 发生错误: {}", node_id, error);
                    }
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