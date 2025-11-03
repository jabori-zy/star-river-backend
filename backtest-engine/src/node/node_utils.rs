// third-party
use tokio::sync::broadcast;

// workspace crate
use event_center::event::{
    node_event::backtest_node_event::BacktestNodeEvent,
    strategy_event::NodeStateLogEvent,
};
use star_river_core::{
    custom_type::{HandleId, NodeId, StrategyId},
    error::StarRiverErrorTrait,
};

// current crate
use super::{
    node_handles::NodeOutputHandle,
    node_state_machine::NodeRunState,
};

pub struct NodeUtils;

impl NodeUtils {
    pub async fn send_success_status_event(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: String,
        msg: String,
        state: String,
        action: String,
        strategy_output_handle: &NodeOutputHandle,
    ) {
        let log_event = NodeStateLogEvent::success(strategy_id, node_id, node_name, state, action, msg);
        let _ = strategy_output_handle.send(log_event.into());
    }

    pub async fn send_error_status_event(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: String,
        action: String,
        error: &impl StarRiverErrorTrait,
        strategy_output_handle: &NodeOutputHandle,
    ) {
        let log_event = NodeStateLogEvent::error(strategy_id, node_id, node_name, NodeRunState::Failed.to_string(), action, error);
        let _ = strategy_output_handle.send(log_event.into());
    }

    pub fn generate_strategy_output_handle(node_id: &NodeId) -> NodeOutputHandle {
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        let strategy_output_handle = NodeOutputHandle::new(node_id.clone(), strategy_output_handle_id, tx);
        strategy_output_handle
    }

    pub fn generate_default_output_handle_id(node_id: &NodeId) -> HandleId {
        format!("{}_default_output", node_id)
    }
}
