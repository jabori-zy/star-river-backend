
use crate::*;
use tokio::sync::RwLock;
use std::sync::Arc;



// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
pub struct StartNodeState { 
    pub node_id: String,
    pub node_name: String,
    pub node_sender: NodeSender,
    
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
    pub fn new(node_id: String, name: String) -> Self {
        StartNode { 
            node_type: NodeType::StartNode,
            node_receivers: vec![],
            state: Arc::new(RwLock::new(StartNodeState {
                node_id: node_id.clone(),
                node_name: name,
                node_sender: NodeSender::new(node_id, broadcast::channel::<NodeMessage>(100).0),
            })),
        }
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

    async fn get_sender(&self) -> NodeSender {
        self.state.read().await.node_sender.clone()
    }

    fn push_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("StartNode run");
        Ok(())
    }
}