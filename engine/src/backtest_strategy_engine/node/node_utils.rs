use super::node_state_machine::BacktestNodeRunState;
use super::node_handles::NodeOutputHandle;
use event_center::event::strategy_event::NodeStateLogEvent;
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::error::StarRiverErrorTrait;

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
        let log_event = NodeStateLogEvent::error(
            strategy_id,
            node_id,
            node_name,
            BacktestNodeRunState::Failed.to_string(),
            action,
            error,
        );
        let _ = strategy_output_handle.send(log_event.into());
    }
}
