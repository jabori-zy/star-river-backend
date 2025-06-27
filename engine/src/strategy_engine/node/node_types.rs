use tokio::sync::broadcast;
use std::fmt::Debug;
use tokio::sync::broadcast::error::SendError;
use std::error::Error;
use serde::{Deserialize, Serialize};
use types::strategy::node_event::NodeEvent;
use sea_orm::prelude::*;
use strum_macros::Display;
use std::str::FromStr;





// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq)]
pub enum NodeType {
    #[strum(serialize = "start_node")]
    StartNode,
    #[strum(serialize = "kline_node")]
    KlineNode,
    #[strum(serialize = "indicator_node")]
    IndicatorNode,
    #[strum(serialize = "if_else_node")]
    IfElseNode,
    #[strum(serialize = "futures_order_node")]
    FuturesOrderNode,
    #[strum(serialize = "position_node")]
    PositionNode,
    #[strum(serialize = "position_management_node")]
    PositionManagementNode,
    #[strum(serialize = "get_variable_node")]
    GetVariableNode,
    #[strum(serialize = "order_node")]
    OrderNode,
    #[strum(serialize = "variable_node")]
    VariableNode,
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
            "kline_node" => Ok(NodeType::KlineNode),
            "indicator_node" => Ok(NodeType::IndicatorNode),
            "if_else_node" => Ok(NodeType::IfElseNode),
            "futures_order_node" => Ok(NodeType::FuturesOrderNode),
            "position_node" => Ok(NodeType::PositionNode),
            "position_management_node" => Ok(NodeType::PositionManagementNode),
            "get_variable_node" => Ok(NodeType::GetVariableNode),
            "order_node" => Ok(NodeType::OrderNode),
            "variable_node" => Ok(NodeType::VariableNode),
            _ => Err(format!("Unknown node type: {}", s))
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum DefaultOutputHandleId {
    #[strum(serialize = "start_node_output")]
    StartNodeOutput,
    #[strum(serialize = "kline_node_output")]
    KlineNodeOutput,
    #[strum(serialize = "indicator_node_output")]
    IndicatorNodeOutput,
    #[strum(serialize = "if_else_node_else_output")]
    IfElseNodeElseOutput,
    #[strum(serialize = "order_node_output")]
    OrderNodeOutput,
    #[strum(serialize = "position_node_update_output")]
    PositionNodeUpdateOutput,
    #[strum(serialize = "get_variable_node_output")]
    GetVariableNodeOutput,

}



#[derive(Debug)]
pub struct NodeInputHandle {
    // 来自哪个节点
    pub from_node_id: String,
    pub from_handle_id: String,
    pub input_handle_id: String, // 对应的input_handle_id
    pub receiver: broadcast::Receiver<NodeEvent>,
}

impl NodeInputHandle {
    pub fn new(
        from_node_id: String,
        from_handle_id: String,
        input_handle_id: String, 
        receiver: broadcast::Receiver<NodeEvent>
    ) -> Self {
        Self { from_node_id, from_handle_id, input_handle_id, receiver }
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<NodeEvent> {
        self.receiver.resubscribe()
    }
}


impl Clone for NodeInputHandle {
    fn clone(&self) -> Self {
        Self { 
            from_node_id: self.from_node_id.clone(), 
            from_handle_id: self.from_handle_id.clone(),
            input_handle_id: self.input_handle_id.clone(),
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
    pub output_handle_id: String,
    pub node_event_sender: broadcast::Sender<NodeEvent>,
    pub connect_count: usize,
}

impl NodeOutputHandle {
    pub fn send(&self, event: NodeEvent) -> Result<usize, String> {
        if self.connect_count > 0 {
            self.node_event_sender.send(event).map_err(|e| format!("节点{}的出口{}发送消息失败: {}", self.node_id, self.output_handle_id, e))
        } else {
            // 如果connect_count为1(默认的一个是连接到策略的)，则不发送消息
            Err(format!("output handle have no connection, node_id:{}, output_handle_id:{}", self.node_id, self.output_handle_id))
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NodeEvent> {
        self.node_event_sender.subscribe()
    }
}

