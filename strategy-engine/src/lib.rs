pub mod strategy;
pub mod node;
pub mod engine;


use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use tokio::sync::broadcast::error::SendError;
use std::error::Error;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use types::strategy::message::NodeMessage;
use sea_orm::prelude::*;
use strum_macros::Display;
use std::str::FromStr;
use crate::node::NodeRunState;

// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum NodeType {
    #[strum(serialize = "start_node")]
    StartNode,
    #[strum(serialize = "live_data_node")]
    LiveDataNode,
    #[strum(serialize = "data_source_node")]
    DataSourceNode,
    #[strum(serialize = "indicator_node")]
    IndicatorNode,
    #[strum(serialize = "if_else_node")]
    IfElseNode,
    #[strum(serialize = "order_node")]
    OrderNode,
}

impl FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 处理指标节点的特殊情况
        if s.ends_with("indicator_node") {
            return Ok(NodeType::IndicatorNode);
        }

        // 其他节点类型保持原有的下划线命名方式
        match s {
            "start_node" => Ok(NodeType::StartNode),
            "live_data_node" => Ok(NodeType::LiveDataNode),
            "data_source_node" => Ok(NodeType::DataSourceNode),
            "indicator_node" => Ok(NodeType::IndicatorNode),
            "if_else_node" => Ok(NodeType::IfElseNode),
            "order_node" => Ok(NodeType::OrderNode),
            _ => Err(format!("Unknown node type: {}", s))
        }
    }
}




#[derive(Debug, Clone)]
pub struct NodeSender {
    pub node_id: String, // 节点id
    pub handle_id: String, // 出口id
    pub sender: broadcast::Sender<NodeMessage>, // 发送者
}

impl NodeSender {
    pub fn new(node_id: String, handle_id: String, sender: broadcast::Sender<NodeMessage>) -> Self {
        Self { node_id, handle_id, sender }
    }
    pub fn subscribe(&self) -> NodeMessageReceiver {
        NodeMessageReceiver::new(self.node_id.clone(), self.sender.subscribe())
    }
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
    pub fn send(&self, message: NodeMessage) -> Result<usize, SendError<NodeMessage>> {
        self.sender.send(message)
    }
}


#[derive(Debug)]
pub struct NodeMessageReceiver {
    // 来自哪个节点
    pub from_node_id: String,
    pub receiver: broadcast::Receiver<NodeMessage>,
}

impl NodeMessageReceiver {
    pub fn new(from_node_id: String, receiver: broadcast::Receiver<NodeMessage>) -> Self {
        Self { from_node_id, receiver }
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<NodeMessage> {
        self.receiver.resubscribe()
    }
}


impl Clone for NodeMessageReceiver {
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

#[derive(Debug, Clone)]
pub struct NodeOutputHandle {
    pub node_id: String,
    pub handle_id: String,
    pub sender: broadcast::Sender<NodeMessage>,
    pub connect_count: usize,
}



















