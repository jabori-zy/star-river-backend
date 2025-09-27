use super::{
    IndicatorNode,
    NodeStateLogEvent,
    BacktestNodeTrait,
    BacktestNodeRunState,
    StarRiverErrorTrait
};


impl IndicatorNode {
    // pub(super) async fn send_success_status_event(&self, msg: String, state: String, action: String) {
    //     let strategy_output_handle = self.get_strategy_output_handle().await;
    //     let log_event = NodeStateLogEvent::success(
    //         self.get_strategy_id().await, 
    //         self.get_node_id().await, 
    //         self.get_node_name().await, 
    //         state, 
    //         action,
    //         msg,
    //         );
    //     let _ = strategy_output_handle.send(log_event.into());
    // }

    // pub(super) async fn send_error_status_event(&self, action: String, error: &impl StarRiverErrorTrait) {
    //     let strategy_output_handle = self.get_strategy_output_handle().await;
    //     let log_event = NodeStateLogEvent::error(
    //         self.get_strategy_id().await, 
    //         self.get_node_id().await, 
    //         self.get_node_name().await, 
    //         BacktestNodeRunState::Failed.to_string(),
    //         action,
    //         error,
    //         );
    //     let _ = strategy_output_handle.send(log_event.into());
    // }
} 