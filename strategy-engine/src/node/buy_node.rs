use crate::NodeType;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::any::Any;
use crate::node::NodeTrait;
use std::error::Error;
use async_trait::async_trait;
use crate::NodeSender;
use crate::NodeReceiver;
use types::strategy::message::NodeMessage;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use futures::stream::{StreamExt, select_all};
use std::collections::HashMap;
use event_center::EventPublisher;
use crate::NodeOutputHandle;
use crate::node::NodeRunState;

#[derive(Debug, Clone)]
pub struct BuyNodeState {
    pub node_id: String,
    pub node_name: String,
    pub buy_value: f64,
    pub node_sender: NodeSender,
    pub node_output_handle: HashMap<String, NodeSender>, // 节点的出口 {handle_id: sender}, 每个handle对应一个sender
    pub node_output_handle1: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher, // 事件发布器
    pub enable_event_publish: bool, // 是否启用事件发布 
}

#[derive(Debug, Clone)]
pub struct BuyNode {
    pub node_type: NodeType,
    pub state: Arc<RwLock<BuyNodeState>>,
    pub node_receivers: Vec<NodeReceiver>,
    pub from_node_id: Vec<String>,
}

impl BuyNode {
    pub fn new(node_id: String, node_name: String, event_publisher: EventPublisher) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        Self {
            node_type: NodeType::BuyNode,
            node_receivers: vec![],
            from_node_id: vec![],
            state: Arc::new(RwLock::new(BuyNodeState {
                node_id: node_id.clone(),
                node_name,
                buy_value: 0.0,
                node_sender: NodeSender::new(node_id, "buy".to_string(), tx),
                node_output_handle: HashMap::new(),
                node_output_handle1: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
            })),
        }
    }

    async fn listen_message(&mut self) {
        let streams: Vec<_> = self.node_receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();
        let mut combined_stream = select_all(streams);

        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(result) = combined_stream.next().await {
                let message = result.unwrap();
                match message {
                    NodeMessage::Signal(signal_message) => {
                        // tracing::debug!("买入节点: {} 收到信号: message={:?}", state.read().await.node_name, signal_message);
                    }
                    _ => {}
                }
            }
        });
    }

    // 获取默认的handle
    pub async fn get_default_handle(state: &Arc<RwLock<BuyNodeState>>) -> NodeSender {
        let state = state.read().await;
        state.node_output_handle.get("buy_node_output").unwrap().clone()
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let buy_node_sender = NodeSender::new(self.state.read().await.node_id.clone(), "buy_node_output".to_string(), tx);
        self.state.write().await.node_output_handle.insert("buy_node_output".to_string(), buy_node_sender);
        self
    }
}

#[async_trait]
impl NodeTrait for BuyNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn get_node_id(&self) -> String {
        self.state.read().await.node_id.clone()
    }

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("buy_node_output").unwrap().clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
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


    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("买入节点开始运行");
        // 创建内部通信通道
        // 启动监听
        self.listen_message().await;

        Ok(())
    }

    async fn get_node_run_state(&self) -> NodeRunState {
        NodeRunState::Running
    }
}

