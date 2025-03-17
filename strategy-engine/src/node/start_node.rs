
use crate::*;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;


// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
pub struct StartNodeState { 
    pub node_id: String,
    pub node_name: String,
    // pub node_sender: NodeSender,
    pub node_output_handle: HashMap<String, NodeSender>, // 节点的出口 {handle_id: sender}, 每个handle对应一个sender
    
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
    pub fn new(node_id: String, node_name: String) -> Self {
        StartNode { 
            node_type: NodeType::StartNode,
            node_receivers: vec![],
            state: Arc::new(RwLock::new(StartNodeState {
                node_id: node_id.clone(),
                node_name: node_name.clone(),
                // node_sender: NodeSender::new(node_id.clone(), "start".to_string(), broadcast::channel::<NodeMessage>(100).0),
                node_output_handle: HashMap::new(),
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
        self.state.write().await.node_output_handle.insert("start_node_output".to_string(), start_node_sender);
        self
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

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("start_node_output").unwrap().clone()
    }


    fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }

    fn add_from_node_id(&mut self, from_node_id: String) {
        
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender) {
        self.state.write().await.node_output_handle.insert(handle_id, sender);
    }

    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("StartNode run");
        Ok(())
    }
}