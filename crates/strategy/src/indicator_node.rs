use types::indicator::Indicators;
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use std::error::Error;
use async_trait::async_trait;
use futures::StreamExt;

use crate::node::*;


// 指标节点
#[derive(Debug, Clone)]
pub struct IndicatorNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub indicator: Indicators,
    pub sender: NodeSender,
    pub receivers: Vec<NodeReceiver>,
}

impl IndicatorNode {
    pub fn new(name: String, indicator: Indicators) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_id = Uuid::new_v4();
        Self { 
            id: node_id, 
            name, 
            node_type: NodeType::Indicator, 
            indicator, 
            sender: NodeSender::new(node_id.to_string(), tx), 
            receivers: Vec::new(),
        }
    }
}

#[async_trait]
impl NodeTrait for IndicatorNode {
    fn id(&self) -> Uuid {
        self.id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }
    fn get_sender(&self) -> NodeSender {
        self.sender.clone()
    }
    fn get_ref_sender(&mut self) -> &mut NodeSender {
        &mut self.sender
    }
    fn push_receiver(&mut self, receiver: NodeReceiver) {
        self.receivers.push(receiver);
    }
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("IndicatorNode run");
        let streams: Vec<_> = self.receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();

        let mut combined_stream = select_all(streams);

        while let Some(receive_message) = combined_stream.next().await {
            if let Ok(receive_message) = receive_message {
                // println!("指标节点{}接收到数据: {:?}", self.name, receive_message);
                let random_number = rand::random::<u16>();
                let message = NodeMessage {
                    from_node_id: self.id,
                    from_node_name: self.name.clone(),
                    value: receive_message.value + random_number as f64,
                    batch_id: receive_message.batch_id,
                    timestamp: receive_message.timestamp,
                    message_type: MessageType::Indicator,
                };
                self.sender.send(message).unwrap();
        }


        }

        Ok(())
    }
}