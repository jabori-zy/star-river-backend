// third-party

// workspace crate

use star_river_core::{
    custom_type::{NodeId, StrategyId},
    error::StarRiverErrorTrait,
};
use strategy_core::{
    event::{log_event::NodeStateLogEvent, node_common_event::CommonEvent},
    node::node_handles::NodeOutputHandle,
};

use super::node_state_machine::NodeRunState;
use crate::node::node_event::BacktestNodeEvent;

// current crate

pub struct NodeUtils;

impl NodeUtils {
    pub async fn send_success_status_event(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: String,
        msg: String,
        state: String,
        action: String,
        strategy_output_handle: &NodeOutputHandle<BacktestNodeEvent>,
    ) {
        let log_event: CommonEvent = NodeStateLogEvent::success(strategy_id, node_id, node_name, state, action, msg).into();
        let _ = strategy_output_handle.send(log_event.into());
    }

    pub async fn send_error_status_event(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: String,
        action: String,
        error: &impl StarRiverErrorTrait,
        strategy_output_handle: &NodeOutputHandle<BacktestNodeEvent>,
    ) {
        let log_event: CommonEvent =
            NodeStateLogEvent::error(strategy_id, node_id, node_name, NodeRunState::Failed.to_string(), action, error).into();
        let _ = strategy_output_handle.send(log_event.into());
    }
}
