pub mod strategy;
pub mod node;
pub mod message;
pub mod engine;


use tokio::sync::broadcast;
use std::fmt::{Debug, Display};
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
use sea_orm::prelude::*;
use strum::EnumString;
use strum_macros::Display;
use std::str::FromStr;


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
    #[strum(serialize = "buy_node")]
    BuyNode,
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
            "buy_node" => Ok(NodeType::BuyNode),
            _ => Err(format!("Unknown node type: {}", s))
        }
    }
}

#[async_trait]
pub trait NodeTrait: Debug + Send + Sync  {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn NodeTrait>;
    async fn get_node_sender(&self, handle_id: String) -> NodeSender;
    async fn get_default_node_sender(&self) -> NodeSender;
    async fn get_node_name(&self) -> String;
    fn add_message_receiver(&mut self, receiver: NodeReceiver); // 添加接收者
    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender); // 添加出口
    fn add_from_node_id(&mut self, from_node_id: String); // 添加from_node_id

    async fn run(&mut self) -> Result<(), Box<dyn Error>>;
}

impl Clone for Box<dyn NodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
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



















