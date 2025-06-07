mod get_variable_node_context;
mod get_variable_node_state_machine;

use crate::strategy_engine::node::node_context::{BacktestNodeContextTrait,BacktestBaseNodeContext};
use std::sync::Arc;
use tokio::sync::RwLock;
use get_variable_node_context::GetVariableNodeContext;
use get_variable_node_state_machine::{GetVariableNodeStateAction,GetVariableNodeStateMachine};
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType,NodeOutputHandle};
use crate::strategy_engine::node::node_state_machine::BacktestNodeStateTransitionEvent;
use std::any::Any;
use async_trait::async_trait;
use std::time::Duration;
use types::strategy::node_event::NodeEvent;
use event_center::EventPublisher;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use types::node::get_variable_node::*;
use tokio::sync::broadcast;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use tokio::sync::Mutex;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use virtual_trading::VirtualTradingSystem;
use crate::strategy_engine::node::node_types::DefaultOutputHandleId;


#[derive(Debug, Clone)]
pub struct GetVariableNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}


impl GetVariableNode {
    pub fn new(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        backtest_config: GetVariableNodeBacktestConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        heartbeat: Arc<Mutex<Heartbeat>>,
        database: DatabaseConnection,
        strategy_command_sender: NodeCommandSender,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Self {
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::GetVariableNode,
            event_publisher,
            vec![response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(GetVariableNodeStateMachine::new(node_id, node_name)),
            strategy_command_sender,
            strategy_inner_event_receiver,
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(GetVariableNodeContext {
                base_context,
                backtest_config,
                heartbeat,
                database,
                virtual_trading_system,
            }))),
        }
        
    }
}

#[async_trait]
impl BacktestNodeTrait for GetVariableNode {
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
        let context = self.get_context();
        let mut state_guard = context.write().await;
        if let Some(get_variable_node_context) = state_guard.as_any_mut().downcast_mut::<GetVariableNodeContext>() {
            let variable_config = get_variable_node_context.backtest_config.variables.clone();
            
            for variable in variable_config {
                let (tx, _) = broadcast::channel::<NodeEvent>(100);
                let output_handle = NodeOutputHandle {
                    node_id: node_id.clone(),
                    output_handle_id: format!("get_variable_node_output_{}", variable.variable.to_string()),
                    node_event_sender: tx,
                    connect_count: 0,
                };

                get_variable_node_context.get_all_output_handle_mut().insert(format!("get_variable_node_output_{}", variable.variable.to_string()), output_handle);
                tracing::debug!("{}: 设置节点出口: {}", node_id, format!("get_variable_node_output_{}", variable.variable.to_string()));
            }

            // 添加一个默认出口，向策略发送消息
            let (tx, _) = broadcast::channel::<NodeEvent>(100);
            let default_output_handle_id = DefaultOutputHandleId::GetVariableNodeOutput;
            let output_handle = NodeOutputHandle {
                node_id: node_id.clone(),
                output_handle_id: default_output_handle_id.to_string(),
                node_event_sender: tx,
                connect_count: 0,
            };

            get_variable_node_context.get_all_output_handle_mut().insert(default_output_handle_id.to_string(), output_handle);
            tracing::debug!("{}: 所有的输出句柄: {:?}", node_id, get_variable_node_context.get_all_output_handle());

        }
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
        self.cancel_task().await.unwrap();
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

        tracing::debug!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(get_variable_node_state_action) = action.as_any().downcast_ref::<GetVariableNodeStateAction>() {
                match get_variable_node_state_action {
                    GetVariableNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    GetVariableNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    GetVariableNodeStateAction::RegisterTask => {
                        tracing::info!("{}: 开始注册任务", node_id);
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(get_variable_node_context) = state_guard.as_any_mut().downcast_mut::<GetVariableNodeContext>() {
                            let backtest_config = get_variable_node_context.backtest_config.clone();
                            let get_variable_type = backtest_config.get_variable_type.clone();
                            // 如果获取变量类型为定时触发，则注册任务
                            if let GetVariableType::Timer = get_variable_type {
                                get_variable_node_context.register_task().await;
                            }
                        }
                    }
                    GetVariableNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_node_events().await?;
                    }
                    GetVariableNodeStateAction::ListenAndHandleStrategyInnerEvents => {
                        tracing::info!("{}: 开始监听策略内部消息", node_id);
                        self.listen_strategy_inner_events().await?;
                    }
                    GetVariableNodeStateAction::LogError(error) => {
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

