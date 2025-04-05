pub mod state_machine;
pub mod start_node_state;

use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use event_center::EventPublisher;
use crate::NodeOutputHandle;
use crate::node::NodeTrait;
use crate::node::start_node::state_machine::StartNodeStateMachine;
use crate::node::NodeStateTransitionEvent;
use crate::node::start_node::state_machine::StartNodeStateAction;
use std::time::Duration;
use crate::node::node_context::{BaseNodeContext, Context};
use std::any::Any;
use event_center::Event;
use crate::*;

use crate::node::start_node::start_node_state::StartNodeState;
use crate::node::state_machine::NodeStateMachine;

// 将需要共享的状态提取出来


#[derive(Debug)]
pub struct StartNode {
    pub state: Arc<RwLock<Box<dyn Context>>>,
    pub node_type: NodeType,
    
}

impl Clone for StartNode {
    fn clone(&self) -> Self {
        StartNode { 
            node_type: self.node_type.clone(), 
            state: self.state.clone(),
        }
    }
}

impl StartNode {
    pub fn new(
        strategy_id: i32,
        node_id: String, 
        node_name: String, 
        event_publisher: EventPublisher,
    ) -> Self {
        let base_state = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            event_publisher,
            vec![],
            Box::new(StartNodeStateMachine::new(NodeRunState::Created, node_id.clone(), node_name.clone())),
        );
        StartNode { 
            node_type: NodeType::StartNode,
            state: Arc::new(RwLock::new(Box::new(StartNodeState {
                base_state,
            }))),
        }
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    // 初始化节点发送者
    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_output_handle = NodeOutputHandle {
            node_id: self.state.read().await.get_node_id().clone(),
            handle_id: "start_node_output".to_string(),
            sender: tx,
            connect_count: 0,
        };
        self.state.write().await.get_output_handle_mut().insert("start_node_output".to_string(), node_output_handle.clone());
        self
    }

    async fn update_run_state(state: Arc<RwLock<Box<dyn Context>>>, event: NodeStateTransitionEvent) -> Result<(), Box<dyn Error>> {
        let node_id = state.read().await.get_node_id().clone();
        let (transition_result, state_manager) = {
            let node_guard = state.read().await;  // 使用读锁获取当前状态
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
                    let current_state = state.read().await.get_state_machine().current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                }
                StartNodeStateAction::LogNodeState => {
                    let current_state = state.read().await.get_state_machine().current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }
                
                _ => {}
            }
            // 更新状态
            {
                let mut state_guard = state.write().await;
                state_guard.set_state_machine(state_manager.clone_box());
            }
        }
    }
        Ok(())
    }

    async fn cancel_task(state: Arc<RwLock<Box<dyn Context>>>) {
        let state_guard = state.read().await;
        state_guard.get_cancel_token().cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", state_guard.get_node_id(), state_guard.get_state_machine().current_state());
    }

    async fn process_event(state: Arc<RwLock<Box<dyn Context>>>, event: Event) {
        tracing::info!("{}: 收到事件: {:?}", state.read().await.get_node_id(), event);
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
    async fn get_context(&self) -> Arc<RwLock<Box<dyn Context>>> {
        self.state.clone()
    }
    // 获取节点id
    async fn get_node_id(&self) -> String {
        self.state.read().await.get_node_id().clone()
    }
    // 获取节点名称
    async fn get_node_name(&self) -> String {
        self.state.read().await.get_node_name().clone()
    }
    // 获取节点运行状态
    async fn get_run_state(&self) -> NodeRunState {
        self.state.read().await.get_run_state()
    }
    // 获取节点状态机
    async fn get_state_machine(&self) -> Box<dyn NodeStateMachine> {
        self.state.read().await.get_state_machine()
    }


    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver) {
        self.state.write().await.get_message_receivers_mut().push(receiver);
    }


    async fn add_from_node_id(&mut self, from_node_id: String) {    
    }

    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>) {
        let node_output_handle = NodeOutputHandle {
            node_id: self.state.read().await.get_node_id().clone(),
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        };
        self.state.write().await.get_output_handle_mut().insert(handle_id.clone(), node_output_handle.clone());
    }

    async fn add_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.get_output_handle_mut().get_mut(&handle_id).unwrap().connect_count += 1;
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.state.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.state.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::InitializeComplete).await.unwrap();
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始启动", state.read().await.get_node_id());
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Stop).await.unwrap();
        
        // 等待所有任务结束
        Self::cancel_task(state.clone()).await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn enable_node_event_push(&mut self) {
    }

    async fn disable_node_event_push(&mut self) {

    }

    // async fn listen_external_events(&self) -> Result<(), String> {
    //     NodeFunction::listen_external_event(
    //         self.state.clone(),
    //         |state| &state.base_state.event_receivers,
    //         |state| &state.base_state.cancel_token,
    //         |state| &state.base_state.node_id,
    //         |event, state| {
    //             Box::pin(async move {
    //                 Self::process_event(state.clone(), event).await;
    //             })
    //         },

    //     ).await;
    //     Ok(())
    // }

    async fn listen_message(&self) -> Result<(), String> {
        Ok(())
    }

    async fn cancel_task(&self) -> Result<(), String> {
        Ok(())
    }
}