use tokio::sync::broadcast;
use std::fmt::Debug;
use tokio::sync::broadcast::error::SendError;
use std::error::Error;
use serde::{Deserialize, Serialize};
use types::strategy::message::NodeMessage;
use sea_orm::prelude::*;
use strum_macros::Display;
use std::str::FromStr;



#[derive(Debug, Clone, PartialEq)]
pub enum NodeRunState {
    Created,        // 节点已创建但未初始化
    Initializing,   // 节点正在初始化
    Ready,          // 节点已初始化，准备好但未运行
    Starting,       // 节点正在启动
    Running,        // 节点正在运行
    Stopping,       // 节点正在停止
    Stopped,        // 节点已停止
    Failed,         // 节点发生错误
}


// 状态转换事件
#[derive(Debug)]
pub enum NodeStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Start,          // 启动节点
    StartComplete,  // 启动完成 -> 进入Running状态
    Stop,           // 停止节点
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 节点失败，带有错误信息
}

// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum NodeType {
    #[strum(serialize = "start_node")]
    StartNode,
    #[strum(serialize = "live_data_node")]
    LiveDataNode,
    #[strum(serialize = "indicator_node")]
    IndicatorNode,
    #[strum(serialize = "if_else_node")]
    IfElseNode,
    #[strum(serialize = "order_node")]
    OrderNode,
    #[strum(serialize = "position_node")]
    PositionNode,
    #[strum(serialize = "get_variable_node")]
    GetVariableNode,
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
            "indicator_node" => Ok(NodeType::IndicatorNode),
            "if_else_node" => Ok(NodeType::IfElseNode),
            "order_node" => Ok(NodeType::OrderNode),
            "position_node" => Ok(NodeType::PositionNode),
            "get_variable_node" => Ok(NodeType::GetVariableNode),
            _ => Err(format!("Unknown node type: {}", s))
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum DefaultOutputHandleId {
    #[strum(serialize = "start_node_output")]
    StartNodeOutput,
    #[strum(serialize = "live_data_node_output")]
    LiveDataNodeOutput,
    #[strum(serialize = "indicator_node_output")]
    IndicatorNodeOutput,
    #[strum(serialize = "if_else_node_else_output")]
    IfElseNodeElseOutput,
    #[strum(serialize = "order_node_output")]
    OrderNodeOutput,
    #[strum(serialize = "position_node_output")]
    PositionNodeOutput,
    #[strum(serialize = "get_variable_node_output")]
    GetVariableNodeOutput,

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