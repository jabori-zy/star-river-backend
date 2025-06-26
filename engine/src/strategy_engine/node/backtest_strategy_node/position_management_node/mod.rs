mod position_node_context;
pub mod position_node_types;
mod position_node_state_machine;

use crate::strategy_engine::node::node_context::{NodeContextTrait,BaseNodeContext};
use std::sync::Arc;
use tokio::sync::RwLock;
use types::strategy::TradeMode;
use position_node_types::*;
use event_center::EventPublisher;
use event_center::Event;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use crate::exchange_engine::ExchangeEngine;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use position_node_state_machine::{PositionNodeStateMachine,PositionNodeStateAction};
use position_node_context::PositionNodeContext;
use crate::strategy_engine::node::{NodeTrait,NodeType};
use crate::strategy_engine::node::node_state_machine::NodeStateTransitionEvent;
use std::any::Any;
use async_trait::async_trait;
use std::time::Duration;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};

#[derive(Debug, Clone)]
pub struct PositionNode {
    pub context: Arc<RwLock<Box<dyn NodeContextTrait>>>,
}


impl PositionNode {
    pub fn new(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        live_config: PositionNodeLiveConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
    ) -> Self {
        let base_context = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::PositionNode,
            event_publisher,
            vec![response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(PositionNodeStateMachine::new(node_id, node_name)),
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(PositionNodeContext {
                base_context,
                live_config,
                exchange_engine,
                database,
                heartbeat,
            }))),
        }
        
    }
}

#[async_trait]
impl NodeTrait for PositionNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn NodeContextTrait>>> {
        self.context.clone()
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.get_node_name().await);
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_node_state(NodeStateTransitionEvent::Initialize).await.unwrap();

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("{:?}: 初始化完成", self.get_state_machine().await.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(NodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始启动", self.get_node_id().await);
        self.update_node_state(NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        self.update_node_state(NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
        
    }

    async fn stop(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(NodeStateTransitionEvent::Stop).await.unwrap();

        // 等待所有任务结束
        self.cancel_task().await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn update_node_state(&mut self, event: NodeStateTransitionEvent) -> Result<(), String> {
        let node_id = self.get_node_id().await;

        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.get_state_machine().await;
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };


        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(position_node_state_action) = action.as_any().downcast_ref::<PositionNodeStateAction>() {
                match position_node_state_action {
                    PositionNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    PositionNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    PositionNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    PositionNodeStateAction::RegisterHeartbeatTask => {
                        tracing::info!("{}: 开始注册心跳任务", node_id);
                        let mut context_guard = self.context.write().await;
                        let position_node_context = context_guard.as_any_mut().downcast_mut::<PositionNodeContext>().unwrap();
                        // position_node_context.monitor_unfilled_order().await;
                    }
                    PositionNodeStateAction::ListenAndHandleMessage => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_message().await?;
                    }
                    PositionNodeStateAction::LogError(error) => {
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