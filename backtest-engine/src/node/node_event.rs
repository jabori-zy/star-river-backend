// ============================================================================
// Import all node event types
// ============================================================================

use derive_more::From;
use serde::{Deserialize, Serialize};
pub use star_river_event::backtest_strategy::node_event::{
    futures_order_node_event::FuturesOrderNodeEvent, if_else_node_event::IfElseNodeEvent, indicator_node_event::IndicatorNodeEvent,
    kline_node_event::KlineNodeEvent, position_node_event::PositionManagementNodeEvent, start_node_event::StartNodeEvent,
    variable_node_event::VariableNodeEvent,
};
use strategy_core::event::node::NodeEventTrait;
pub use strategy_core::event::node_common_event::CommonEvent;
use strum::Display;

// ============================================================================
// Backtest Node Event Unified Enum
// ============================================================================

/// Backtest node event unified enum
///
/// Wraps all types of node events and provides a unified event interface
#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "node_type")]
pub enum BacktestNodeEvent {
    #[strum(serialize = "start_node")]
    #[serde(rename = "start_node")]
    StartNode(StartNodeEvent),

    #[strum(serialize = "indicator_node")]
    #[serde(rename = "indicator_node")]
    IndicatorNode(IndicatorNodeEvent),

    #[strum(serialize = "common")]
    #[serde(rename = "common")]
    Common(CommonEvent),

    #[strum(serialize = "variable_node")]
    #[serde(rename = "variable_node")]
    VariableNode(VariableNodeEvent),

    #[strum(serialize = "kline_node")]
    #[serde(rename = "kline_node")]
    KlineNode(KlineNodeEvent),

    #[strum(serialize = "futures_order_node")]
    #[serde(rename = "futures_order_node")]
    FuturesOrderNode(FuturesOrderNodeEvent),

    #[strum(serialize = "position_management_node")]
    #[serde(rename = "position_management_node")]
    PositionManagementNode(PositionManagementNodeEvent),

    #[strum(serialize = "if_else_node")]
    #[serde(rename = "if_else_node")]
    IfElseNode(IfElseNodeEvent),
}

impl NodeEventTrait for BacktestNodeEvent {}
