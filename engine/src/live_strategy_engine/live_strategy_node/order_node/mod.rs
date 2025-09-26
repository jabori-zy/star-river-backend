mod order_node_context;
mod order_node_state_machine;
pub mod order_node_types;

use super::order_node::order_node_context::OrderNodeContext;
use super::order_node::order_node_state_machine::{OrderNodeStateAction, OrderNodeStateMachine};
use crate::exchange_engine::ExchangeEngine;
use crate::strategy_engine::node::node_context::{LiveBaseNodeContext, LiveNodeContextTrait};
use crate::strategy_engine::node::node_state_machine::LiveNodeStateTransitionEvent;
use crate::strategy_engine::node::{LiveNodeTrait, NodeType};
use async_trait::async_trait;
use event_center::event::Event;
use event_center::EventPublisher;
use event_center::{CommandPublisher, EngineCommandReceiver, EventReceiver};
use heartbeat::Heartbeat;
use order_node_types::*;
use sea_orm::DatabaseConnection;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use star_river_core::strategy::node_command::NodeCommandSender;

#[derive(Debug, Clone)]
pub struct OrderNode {
    pub context: Arc<RwLock<Box<dyn LiveNodeContextTrait>>>,
}

impl OrderNode {
    pub fn new(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        live_config: OrderNodeLiveConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        exchange_engine: Arc<Mutex<ExchangeEngine>>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_sender: NodeCommandSender,
    ) -> Self {
        let base_context = LiveBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::OrderNode,
            event_publisher,
            vec![response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(OrderNodeStateMachine::new(node_id, node_name)),
            strategy_command_sender,
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(OrderNodeContext {
                base_context,
                live_config,
                request_id: vec![],
                is_processing_order: Arc::new(RwLock::new(false)),
                exchange_engine,
                database,
                heartbeat,
                unfilled_order: Arc::new(RwLock::new(None)),
            }))),
        }
    }
}

#[async_trait]
impl LiveNodeTrait for OrderNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn LiveNodeTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn LiveNodeContextTrait>>> {
        self.context.clone()
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!(
            "================={}====================",
            self.get_node_name().await
        );
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_node_state(LiveNodeStateTransitionEvent::Initialize)
            .await
            .unwrap();

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!(
            "{:?}: 初始化完成",
            self.get_state_machine().await.current_state()
        );
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(LiveNodeStateTransitionEvent::InitializeComplete)
            .await?;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始启动", self.get_node_id().await);
        self.update_node_state(LiveNodeStateTransitionEvent::Start)
            .await
            .unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        self.update_node_state(LiveNodeStateTransitionEvent::StartComplete)
            .await
            .unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(LiveNodeStateTransitionEvent::Stop)
            .await
            .unwrap();

        // 等待所有任务结束
        self.cancel_task().await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(LiveNodeStateTransitionEvent::StopComplete)
            .await
            .unwrap();
        Ok(())
    }

    async fn update_node_state(
        &mut self,
        event: LiveNodeStateTransitionEvent,
    ) -> Result<(), String> {
        let node_id = self.get_node_id().await;

        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.get_state_machine().await;
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };

        tracing::debug!(
            "{}需要执行的动作: {:?}",
            node_id,
            transition_result.get_actions()
        );

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            // 克隆actions避免移动问题
            if let Some(order_node_state_action) =
                action.as_any().downcast_ref::<OrderNodeStateAction>()
            {
                match order_node_state_action {
                    OrderNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!(
                            "{}: 状态转换: {:?} -> {:?}",
                            node_id,
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    OrderNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    OrderNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    OrderNodeStateAction::RegisterTask => {
                        tracing::info!("{}: 开始注册心跳任务", node_id);
                        let mut context_guard = self.context.write().await;
                        let order_node_context = context_guard
                            .as_any_mut()
                            .downcast_mut::<OrderNodeContext>()
                            .unwrap();
                        order_node_context.monitor_unfilled_order().await;
                    }
                    OrderNodeStateAction::ListenAndHandleMessage => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_message().await?;
                    }
                    OrderNodeStateAction::LogError(error) => {
                        tracing::error!("{}: 发生错误: {}", node_id, error);
                    }
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
