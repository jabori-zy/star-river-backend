pub mod state_manager;

use crate::*;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use event_center::EventPublisher;
use crate::NodeOutputHandle;
use crate::node::NodeTrait;
use crate::node::start_node::state_manager::StartNodeStateManager;
use tokio_util::sync::CancellationToken;
use crate::node::NodeStateTransitionEvent;
use crate::node::start_node::state_manager::StartNodeStateAction;
use std::time::Duration;

// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
pub struct StartNodeState { 
    pub node_id: String,
    pub node_name: String,
    // pub node_sender: NodeSender,
    pub node_output_handle: HashMap<String, NodeSender>, // 节点的出口 {handle_id: sender}, 每个handle对应一个sender
    pub node_output_handle1: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher, // 事件发布器
    pub enable_event_publish: bool, // 是否启用事件发布
    pub run_state_manager: StartNodeStateManager,
    pub cancel_token: CancellationToken,
    pub from_node_id: Vec<String>,
    
}

#[derive(Debug)]
pub struct StartNode {
    pub state: Arc<RwLock<StartNodeState>>,
    pub node_receivers: Vec<NodeReceiver>,
    pub node_type: NodeType,
    
}

impl Clone for StartNode {
    fn clone(&self) -> Self {
        StartNode { 
            node_receivers: self.node_receivers.clone(),
            node_type: self.node_type.clone(), 
            state: self.state.clone(),
        }
    }
}

impl StartNode {
    pub fn new(
        node_id: String, 
        node_name: String, 
        event_publisher: EventPublisher,
    ) -> Self {
        StartNode { 
            node_type: NodeType::StartNode,
            node_receivers: vec![],
            state: Arc::new(RwLock::new(StartNodeState {
                node_id: node_id.clone(),
                node_name: node_name.clone(),
                // node_sender: NodeSender::new(node_id.clone(), "start".to_string(), broadcast::channel::<NodeMessage>(100).0),
                node_output_handle: HashMap::new(),
                node_output_handle1: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
                run_state_manager: StartNodeStateManager::new(NodeRunState::Created, node_id, node_name),
                cancel_token: CancellationToken::new(),
                from_node_id: vec![],
            })),
        }
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    // 初始化节点发送者
    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let start_node_sender = NodeSender::new(self.state.read().await.node_id.clone(), "start_node_output".to_string(), tx);
        self.state.write().await.node_output_handle.insert("start_node_output".to_string(), start_node_sender.clone());
        self.state.write().await.node_output_handle1.insert("start_node_output".to_string(), NodeOutputHandle {
            handle_id: "start_node_output".to_string(),
            sender: start_node_sender.clone(),
            connect_count: 0,
        });
        self
    }

    async fn update_run_state(state: Arc<RwLock<StartNodeState>>, event: NodeStateTransitionEvent) -> Result<(), Box<dyn Error>> {
        let node_id = state.read().await.node_id.clone();
        let (transition_result, state_manager) = {
            let node_guard = state.read().await;  // 使用读锁获取当前状态
            let mut state_manager = node_guard.run_state_manager.clone();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };
        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.actions);
        // 执行转换后需要执行的动作
        for action in transition_result.actions.clone() {  // 克隆actions避免移动问题
            match action {
                StartNodeStateAction::LogTransition => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.new_state);
                }
                StartNodeStateAction::LogNodeState => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }
                
                _ => {}
            }
            // 更新状态
            {
                let mut state_guard = state.write().await;
                state_guard.run_state_manager = state_manager.clone();
            }
        }
        Ok(())
    }

    async fn cancel_task(state: Arc<RwLock<StartNodeState>>) {
        let state_guard = state.read().await;
        state_guard.cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", state_guard.node_id, state_guard.run_state_manager.current_state());
    }

}

#[async_trait]
impl NodeTrait for StartNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }
    async fn get_node_id(&self) -> String {
        self.state.read().await.node_id.clone()
    }

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("start_node_output").unwrap().clone()
    }


    async fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.state.write().await.from_node_id.push(from_node_id);
        
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender) {
        self.state.write().await.node_output_handle.insert(handle_id.clone(), sender.clone());
        self.state.write().await.node_output_handle1.insert(handle_id.clone(), NodeOutputHandle {
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        });
    }

    async fn add_node_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.node_output_handle1.get_mut(&handle_id).unwrap().connect_count += 1;
    }

    async fn enable_node_event_publish(&mut self) {
        self.state.write().await.enable_event_publish = true;
    }

    async fn disable_node_event_publish(&mut self) {
        self.state.write().await.enable_event_publish = false;
    }
    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.node_name);
        tracing::info!("{}: 开始初始化", self.state.read().await.node_name);
        // 开始初始化 created -> Initialize
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.state.read().await.run_state_manager.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::InitializeComplete).await.unwrap();
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始启动", state.read().await.node_id);
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.node_id);
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Stop).await.unwrap();
        
        // 等待所有任务结束
        Self::cancel_task(state.clone()).await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }


    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.run_state_manager.current_state()
    }
}