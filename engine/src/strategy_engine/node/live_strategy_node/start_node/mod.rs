pub mod start_node_state_machine;
pub mod start_node_context;

use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use crate::strategy_engine::node::node_state_machine::NodeStateTransitionEvent;
use crate::strategy_engine::node::{NodeTrait,NodeType};
use crate::strategy_engine::node::node_context::{BaseNodeContext, NodeContextTrait};
use super::start_node::start_node_state_machine::{StartNodeStateMachine,StartNodeStateAction};
use std::time::Duration;
use std::any::Any;
use crate::*;
use super::start_node::start_node_context::StartNodeContext;
use types::strategy::{LiveStrategyConfig, BacktestStrategyConfig, SimulatedConfig, TradeMode};
use event_center::{CommandPublisher, CommandReceiver};
use types::strategy::node_command::NodeCommandSender;

#[derive(Debug)]
pub struct StartNode {
    pub context: Arc<RwLock<Box<dyn NodeContextTrait>>>
}

impl Clone for StartNode {
    fn clone(&self) -> Self {
        StartNode {
            context: self.context.clone(),
        }
    }
}

impl StartNode {
    pub fn new(
        strategy_id: i32,
        node_id: String, 
        node_name: String,
        live_config: LiveStrategyConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        strategy_command_sender: NodeCommandSender,
    ) -> Self {
        let base_context = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::StartNode,
            event_publisher,
            vec![],
            command_publisher,
            command_receiver,
            Box::new(StartNodeStateMachine::new(node_id.clone(), node_name.clone())),
            strategy_command_sender,
        );
        StartNode {
            context: Arc::new(RwLock::new(Box::new(StartNodeContext {
                base_context,
                live_config,
            }))),
        }
    }

}

#[async_trait]
impl NodeTrait for StartNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    // get方法
    // 获取节点上下文
    fn get_context(&self) -> Arc<RwLock<Box<dyn NodeContextTrait>>> {
        self.context.clone()
    }


    async fn add_from_node_id(&mut self, from_node_id: String) {
        let _from_node_id = from_node_id;
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(NodeStateTransitionEvent::InitializeComplete).await.unwrap();
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        let state = self.context.clone();
        tracing::info!("{}: 开始启动", state.read().await.get_node_id());
        self.update_node_state(NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        self.update_node_state(NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(NodeStateTransitionEvent::Stop).await.unwrap();
        
        // 等待所有任务结束
        self.cancel_task().await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn enable_node_event_push(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn disable_node_event_push(&mut self) -> Result<(), String> {
        Ok(())
    }

    async fn listen_message(&self) -> Result<(), String> {
        Ok(())
    }

    async fn update_node_state(&mut self, event: NodeStateTransitionEvent) -> Result<(), String> {
        let node_id = self.context.read().await.get_node_id().clone();
        let (transition_result, state_manager) = {
            let node_guard = self.context.read().await;  // 使用读锁获取当前状态
            let mut state_manager = node_guard.get_state_machine().clone_box();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };
        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(start_action) = action.as_any().downcast_ref::<StartNodeStateAction>() {
                match start_action {
                StartNodeStateAction::LogTransition => {
                    let current_state = self.context.read().await.get_state_machine().current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                }
                StartNodeStateAction::LogNodeState => {
                    let current_state = self.context.read().await.get_state_machine().current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }

                _ => {}
            }
            // 更新状态
            {
                let mut state_guard = self.context.write().await;
                state_guard.set_state_machine(state_manager.clone_box());
            }
        }
    }
        Ok(())
    }
}