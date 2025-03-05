pub mod strategy;
pub mod data_source_node;
pub mod indicator_node;
pub mod condition_node;
pub mod message;


use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast::error::SendError;
use std::error::Error;
use async_trait::async_trait;
use types::market::KlineSeries;
use types::indicator::{Indicators, IndicatorData};
use serde::{Deserialize, Serialize};
use crate::message::NodeMessage;






// 节点类型
#[derive(Debug, Clone)]
pub enum NodeType {
    Start,
    DataSource,
    Indicator,
    Condition,
}

#[async_trait]
pub trait NodeTrait: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn NodeTrait>;
    async fn get_sender(&self) -> NodeSender;
    fn push_receiver(&mut self, receiver: NodeReceiver);
    async fn run(&mut self) -> Result<(), Box<dyn Error>>;
}

impl Clone for Box<dyn NodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


// 开始节点
#[derive(Debug, Clone)]
pub struct StartNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub sender: NodeSender,
    pub receivers: Vec<NodeReceiver>,
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
        self.sender.clone()
    }

    fn push_receiver(&mut self, receiver: NodeReceiver) {
        self.receivers.push(receiver);
    }
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct NodeSender {
    pub node_id: String,
    pub sender: broadcast::Sender<NodeMessage>,
}

impl NodeSender {
    pub fn new(node_id: String, sender: broadcast::Sender<NodeMessage>) -> Self {
        Self { node_id, sender }
    }
    pub fn subscribe(&self) -> NodeReceiver {
        NodeReceiver::new(self.node_id.clone(), self.sender.subscribe())
    }
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
    pub fn send(&self, message: NodeMessage) -> Result<usize, SendError<NodeMessage>> {
        self.sender.send(message)
    }
}


#[derive(Debug)]
pub struct NodeReceiver {
    // 来自哪个节点
    pub from_node_id: String,
    pub receiver: broadcast::Receiver<NodeMessage>,
}

impl NodeReceiver {
    pub fn new(from_node_id: String, receiver: broadcast::Receiver<NodeMessage>) -> Self {
        Self { from_node_id, receiver }
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<NodeMessage> {
        self.receiver.resubscribe()
    }
}


impl Clone for NodeReceiver {
    fn clone(&self) -> Self {
        Self { 
            from_node_id: self.from_node_id.clone(), 
            receiver: self.receiver.resubscribe()
        }
    }
}





#[derive(Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: NodeType,
    pub target: NodeType,
}
















