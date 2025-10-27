use crate::backtest_strategy_engine::node::node_handles::NodeInputHandle;
use event_center::event::node_event::backtest_node_event::start_node_event::KlinePlayPayload;
use event_center::event::node_event::backtest_node_event::{
    BacktestNodeEvent,
    start_node_event::{KlinePlayEvent, StartNodeEvent},
};
use star_river_core::custom_type::PlayIndex;
use tokio::sync::broadcast;
use tokio::time::Duration;

/// Mock start node for testing
///
/// This mock node simulates the start node in the workflow that sends KlinePlay events.
/// It's used to test the full flow: receive event -> fetch data -> send data
pub struct MockStartNode {
    /// Node ID for identification
    node_id: String,

    /// Output handle ID
    output_handle_id: String,

    /// Output handle for sending events to connected nodes
    output_handle: broadcast::Sender<BacktestNodeEvent>,

    /// Number of times to send events
    send_count: usize,
}

impl MockStartNode {
    /// Create a new mock start node
    ///
    /// # Parameters
    ///
    /// - `send_count`: Number of times to send KlinePlay events
    ///
    /// # Returns
    ///
    /// Returns a tuple of (MockStartNode, NodeInputHandle) where the NodeInputHandle can be added to KlineNode
    pub fn new(send_count: usize) -> (Self, NodeInputHandle) {
        let node_id = "mock_start_node_1".to_string();
        let output_handle_id = "mock_start_node_1_output_1".to_string();

        let (tx, rx) = broadcast::channel(100);

        let node = MockStartNode {
            node_id: node_id.clone(),
            output_handle_id: output_handle_id.clone(),
            output_handle: tx,
            send_count,
        };

        // Create NodeInputHandle for the downstream node
        let input_handle = NodeInputHandle::new(
            node_id,
            output_handle_id,
            "default_input".to_string(), // Input handle ID for the receiving node
            rx,
        );

        (node, input_handle)
    }

    /// Send KlinePlay events according to the configured send_count
    ///
    /// # Returns
    ///
    /// Returns the number of events successfully sent
    pub async fn send_events(&self) -> usize {
        let mut sent = 0;

        for play_index in 0..self.send_count {
            let payload = KlinePlayPayload::new(play_index as PlayIndex);
            let kline_play_event: StartNodeEvent = KlinePlayEvent::new(
                "test_kline_node_1".to_string(),
                "Test Kline Node".to_string(),
                "test_kline_node_1_output_1".to_string(),
                payload,
            ).into();

            match self.output_handle.send(kline_play_event.into()) {
                Ok(receiver_count) => {
                    tracing::debug!(
                        "MockStartNode [{}] sent KlinePlay event (play_index={}) to {} receivers",
                        self.node_id,
                        play_index,
                        receiver_count
                    );
                    sent += 1;
                }
                Err(e) => {
                    tracing::error!(
                        "MockStartNode [{}] failed to send event (play_index={}): {:?}",
                        self.node_id,
                        play_index,
                        e
                    );
                    break;
                }
            }

            // Add small delay between events to avoid overwhelming receivers
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }

        tracing::info!(
            "MockStartNode [{}] sent {} events out of {} configured",
            self.node_id,
            sent,
            self.send_count
        );
        sent
    }

    /// Get the node ID
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    /// Get the output handle ID
    pub fn get_output_handle_id(&self) -> &str {
        &self.output_handle_id
    }
}
