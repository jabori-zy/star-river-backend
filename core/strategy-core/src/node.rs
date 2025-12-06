mod base_node;
pub mod context_trait;
pub mod metadata;
pub mod node_handles;
pub mod node_state_machine;
pub mod node_trait;
pub mod utils;

use std::str::FromStr;

pub use base_node::NodeBase;
pub use node_trait::NodeTrait;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum NodeType {
    StartNode,
    KlineNode,
    IndicatorNode,
    IfElseNode,
    FuturesOrderNode,
    PositionNode,
    VariableNode,
}

impl FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle special case for indicator nodes
        if s.ends_with("indicator_node") || s.ends_with("IndicatorNode") {
            return Ok(NodeType::IndicatorNode);
        }

        // Support both camelCase and snake_case naming conventions
        match s {
            // Snake case format
            "start_node" => Ok(NodeType::StartNode),
            "kline_node" => Ok(NodeType::KlineNode),
            "indicator_node" => Ok(NodeType::IndicatorNode),
            "if_else_node" => Ok(NodeType::IfElseNode),
            "futures_order_node" => Ok(NodeType::FuturesOrderNode),
            "position_node" => Ok(NodeType::PositionNode),
            "variable_node" => Ok(NodeType::VariableNode),
            // Camel case format
            "startNode" => Ok(NodeType::StartNode),
            "klineNode" => Ok(NodeType::KlineNode),
            "indicatorNode" => Ok(NodeType::IndicatorNode),
            "ifElseNode" => Ok(NodeType::IfElseNode),
            "futuresOrderNode" => Ok(NodeType::FuturesOrderNode),
            "positionNode" => Ok(NodeType::PositionNode),
            "variableNode" => Ok(NodeType::VariableNode),
            _ => Err(format!("Unknown node type: {}", s)),
        }
    }
}
