// ============================================================================
// Import all node event types
// ============================================================================

use chrono::{DateTime, Utc};
use derive_more::From;
use serde::Serialize;
use star_river_core::custom_type::{CycleId, HandleId, NodeId, NodeName};
pub use star_river_event::backtest_strategy::node_event::{
    futures_order_node_event::FuturesOrderNodeEvent, if_else_node_event::IfElseNodeEvent, indicator_node_event::IndicatorNodeEvent,
    kline_node_event::KlineNodeEvent, start_node_event::StartNodeEvent, variable_node_event::VariableNodeEvent,
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
#[derive(Debug, Clone, Serialize, Display, From)]
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

    // #[strum(serialize = "position_node")]
    // #[serde(rename = "position_node")]
    // PositionNode(PositionNodeEvent),
    #[strum(serialize = "if_else_node")]
    #[serde(rename = "if_else_node")]
    IfElseNode(IfElseNodeEvent),
}

impl NodeEventTrait for BacktestNodeEvent {
    fn cycle_id(&self) -> CycleId {
        match self {
            BacktestNodeEvent::StartNode(event) => event.cycle_id(),
            BacktestNodeEvent::IndicatorNode(event) => event.cycle_id(),
            BacktestNodeEvent::Common(event) => event.cycle_id(),
            BacktestNodeEvent::VariableNode(event) => event.cycle_id(),
            BacktestNodeEvent::KlineNode(event) => event.cycle_id(),
            BacktestNodeEvent::FuturesOrderNode(event) => event.cycle_id(),
            // BacktestNodeEvent::PositionNode(event) => event.cycle_id(),
            BacktestNodeEvent::IfElseNode(event) => event.cycle_id(),
        }
    }

    fn datetime(&self) -> DateTime<Utc> {
        match self {
            BacktestNodeEvent::StartNode(event) => event.datetime(),
            BacktestNodeEvent::IndicatorNode(event) => event.datetime(),
            BacktestNodeEvent::Common(event) => event.datetime(),
            BacktestNodeEvent::VariableNode(event) => event.datetime(),
            BacktestNodeEvent::KlineNode(event) => event.datetime(),
            BacktestNodeEvent::FuturesOrderNode(event) => event.datetime(),
            // BacktestNodeEvent::PositionNode(event) => event.datetime(),
            BacktestNodeEvent::IfElseNode(event) => event.datetime(),
        }
    }

    fn node_id(&self) -> &NodeId {
        match self {
            BacktestNodeEvent::StartNode(event) => event.node_id(),
            BacktestNodeEvent::IndicatorNode(event) => event.node_id(),
            BacktestNodeEvent::Common(event) => event.node_id(),
            BacktestNodeEvent::VariableNode(event) => event.node_id(),
            BacktestNodeEvent::KlineNode(event) => event.node_id(),
            BacktestNodeEvent::FuturesOrderNode(event) => event.node_id(),
            // BacktestNodeEvent::PositionNode(event) => event.node_id(),
            BacktestNodeEvent::IfElseNode(event) => event.node_id(),
        }
    }
    fn node_name(&self) -> &NodeName {
        match self {
            BacktestNodeEvent::StartNode(event) => event.node_name(),
            BacktestNodeEvent::IndicatorNode(event) => event.node_name(),
            BacktestNodeEvent::Common(event) => event.node_name(),
            BacktestNodeEvent::VariableNode(event) => event.node_name(),
            BacktestNodeEvent::KlineNode(event) => event.node_name(),
            BacktestNodeEvent::FuturesOrderNode(event) => event.node_name(),
            // BacktestNodeEvent::PositionNode(event) => event.node_name(),
            BacktestNodeEvent::IfElseNode(event) => event.node_name(),
        }
    }
    fn output_handle_id(&self) -> &HandleId {
        match self {
            BacktestNodeEvent::StartNode(event) => event.output_handle_id(),
            BacktestNodeEvent::IndicatorNode(event) => event.output_handle_id(),
            BacktestNodeEvent::Common(event) => event.output_handle_id(),
            BacktestNodeEvent::VariableNode(event) => event.output_handle_id(),
            BacktestNodeEvent::KlineNode(event) => event.output_handle_id(),
            BacktestNodeEvent::FuturesOrderNode(event) => event.output_handle_id(),
            // BacktestNodeEvent::PositionNode(event) => event.output_handle_id(),
            BacktestNodeEvent::IfElseNode(event) => event.output_handle_id(),
        }
    }
}
