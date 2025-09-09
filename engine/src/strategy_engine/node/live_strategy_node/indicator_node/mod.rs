pub mod indicator_node_context;
pub mod indicator_node_state_machine;
pub mod indicator_node_type;

use crate::strategy_engine::node::node_context::{LiveBaseNodeContext, LiveNodeContextTrait};
use crate::strategy_engine::node::node_state_machine::*;
use crate::strategy_engine::node::{LiveNodeTrait, NodeType};
use async_trait::async_trait;
use event_center::EventPublisher;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use indicator_node_context::IndicatorNodeContext;
use indicator_node_state_machine::{IndicatorNodeStateAction, IndicatorNodeStateManager};
use indicator_node_type::IndicatorNodeLiveConfig;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use types::strategy::node_command::NodeCommandSender;

// 指标节点
#[derive(Debug, Clone)]
pub struct IndicatorNode {
    pub context: Arc<RwLock<Box<dyn LiveNodeContextTrait>>>,
}

impl IndicatorNode {
    pub fn new(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        live_config: IndicatorNodeLiveConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        strategy_command_sender: NodeCommandSender,
    ) -> Self {
        let base_context = LiveBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IndicatorNode,
            event_publisher,
            vec![response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(IndicatorNodeStateManager::new(
                LiveNodeRunState::Created,
                node_id,
                node_name,
            )),
            strategy_command_sender,
        );

        Self {
            context: Arc::new(RwLock::new(Box::new(IndicatorNodeContext {
                base_context,
                live_config,
                is_registered: Arc::new(RwLock::new(false)),
            }))),
        }
    }
}

#[async_trait]
impl LiveNodeTrait for IndicatorNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn LiveNodeTrait> {
        Box::new(self.clone())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn LiveNodeContextTrait>>> {
        self.context.clone()
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!(
            "================={}====================",
            self.context.read().await.get_node_name()
        );
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(LiveNodeStateTransitionEvent::Initialize)
            .await
            .unwrap();

        // 循环检查是否已经注册指标
        // 检查交易所是否注册成功，并且K线流是否订阅成功
        loop {
            let is_registered = {
                let state_guard = self.context.read().await;
                let indicator_node_context = state_guard
                    .as_any()
                    .downcast_ref::<IndicatorNodeContext>()
                    .unwrap();
                let is_registered = indicator_node_context.is_registered.read().await.clone();
                tracing::info!(
                    "{}: 检查是否已经注册指标: {}",
                    self.get_node_id().await,
                    is_registered
                );
                is_registered
            };
            if is_registered {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        tracing::info!(
            "{:?}: 初始化完成",
            self.context
                .read()
                .await
                .get_state_machine()
                .current_state()
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

        tracing::info!(
            "{}需要执行的动作: {:?}",
            node_id,
            transition_result.get_actions()
        );
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(indicator_node_state_action) =
                action.as_any().downcast_ref::<IndicatorNodeStateAction>()
            {
                match indicator_node_state_action {
                    IndicatorNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!(
                            "{}: 状态转换: {:?} -> {:?}",
                            node_id,
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    IndicatorNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    IndicatorNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandleMessage => {
                        tracing::info!("{}: 开始监听节点传递的message", node_id);
                        self.listen_message().await?;
                    }
                    IndicatorNodeStateAction::RegisterIndicator => {
                        tracing::info!("{}: 开始注册指标", node_id);
                        let mut context = self.context.write().await;
                        let context = context
                            .as_any_mut()
                            .downcast_mut::<IndicatorNodeContext>()
                            .unwrap();
                        let register_indicator_response = context.register_indicator().await;
                        if let Ok(register_indicator_response) = register_indicator_response {
                            if register_indicator_response.success() {
                                *context.is_registered.write().await = true;
                                tracing::info!("{}: 注册指标成功", node_id);
                            } else {
                                tracing::error!(
                                    "{}: 注册指标失败: {:?}",
                                    node_id,
                                    register_indicator_response
                                );
                            }
                        }
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
