use event_center::event::{node_event::backtest_node_event::{kline_node_event::{KlineUpdateEvent, KlineNodeEvent}, BacktestNodeEvent}, node_event::NodeEventTrait};
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use std::collections::HashMap;

/// Mock indicator node for testing
///
/// This mock node simulates an indicator node that subscribes to kline node's output handles.
/// It receives and logs kline events from the upstream kline node.
pub struct MockIndicatorNode {
    /// Node identifier for logging
    node_id: String,

    /// Receivers for kline node events (one for each output handle)
    event_receivers: Vec<Arc<Mutex<broadcast::Receiver<BacktestNodeEvent>>>>,

    /// Background task handles for cleanup
    listen_tasks: Vec<tokio::task::JoinHandle<()>>,

    /// Cache for received KlineUpdate events, grouped by output handle ID (thread-safe)
    /// Key: from_node_handle_id, Value: Vec of KlineUpdateEvents from that handle
    received_kline_update_events: Arc<Mutex<HashMap<String, Vec<KlineUpdateEvent>>>>,
}

impl MockIndicatorNode {
    /// Create a new mock indicator node
    ///
    /// # Parameters
    ///
    /// - `node_id`: Identifier for this mock node (used in logging)
    /// - `event_receivers`: Vector of receivers subscribed to kline node's output handles
    ///
    /// # Returns
    ///
    /// Returns a MockIndicatorNode instance
    pub fn new(node_id: String, event_receivers: Vec<broadcast::Receiver<BacktestNodeEvent>>) -> Self {
        let receivers = event_receivers
            .into_iter()
            .map(|rx| Arc::new(Mutex::new(rx)))
            .collect();

        Self {
            node_id,
            event_receivers: receivers,
            listen_tasks: Vec::new(),
            received_kline_update_events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start listening to events from all output handles
    ///
    /// This spawns a background task for each receiver that continuously listens for events
    /// and logs them when received.
    pub fn start_listening(&mut self) {
        tracing::info!(
            "MockIndicatorNode [{}] starting to listen on {} output handles",
            self.node_id,
            self.event_receivers.len()
        );

        for (index, event_receiver) in self.event_receivers.iter().enumerate() {
            let node_id = self.node_id.clone();
            let receiver = event_receiver.clone();
            let handle_index = index;
            let kline_update_cache = self.received_kline_update_events.clone();

            let task = tokio::spawn(async move {
                tracing::info!(
                    "MockIndicatorNode [{}] started listening on handle #{}",
                    node_id,
                    handle_index
                );

                loop {
                    let mut rx = receiver.lock().await;

                    match rx.recv().await {
                        Ok(event) => {
                            match event {
                                BacktestNodeEvent::KlineNode(kline_event) => {

                                    // Cache KlineUpdate events for verification
                                    if let KlineNodeEvent::KlineUpdate(kline_update_event) = &kline_event {
                                        let from_handle_id = kline_update_event.from_node_handle_id().clone();
                                        let mut cache = kline_update_cache.lock().await;

                                        cache.entry(from_handle_id.clone())
                                            .or_insert_with(Vec::new)
                                            .push(kline_update_event.clone());

                                        let handle_count = cache.get(&from_handle_id).map(|v| v.len()).unwrap_or(0);
                                        let total_count: usize = cache.values().map(|v| v.len()).sum();

                                        tracing::debug!(
                                            "MockIndicatorNode [{}] cached KlineUpdate event from handle '{}', handle count: {}, total count: {}",
                                            node_id,
                                            from_handle_id,
                                            handle_count,
                                            total_count
                                        );
                                    }
                                }
                                _ => {
                                    tracing::debug!(
                                        "MockIndicatorNode [{}] handle #{} received other event: {:?}",
                                        node_id,
                                        handle_index,
                                        event
                                    );
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(
                                "MockIndicatorNode [{}] handle #{} lagged behind, skipped {} events",
                                node_id,
                                handle_index,
                                skipped
                            );
                            continue;
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            tracing::info!(
                                "MockIndicatorNode [{}] handle #{} channel closed, stopping listener",
                                node_id,
                                handle_index
                            );
                            break;
                        }
                    }
                }

                tracing::info!(
                    "MockIndicatorNode [{}] handle #{} stopped listening",
                    node_id,
                    handle_index
                );
            });

            self.listen_tasks.push(task);
        }
    }

    /// Stop listening to events and cleanup resources
    pub async fn stop_listening(&mut self) {
        tracing::info!(
            "MockIndicatorNode [{}] stopping {} listening tasks",
            self.node_id,
            self.listen_tasks.len()
        );

        for task in self.listen_tasks.drain(..) {
            task.abort();
        }

        tracing::debug!("MockIndicatorNode [{}] all listening tasks stopped", self.node_id);
    }

    /// Get the node identifier
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    /// Get the number of subscribed output handles
    pub fn get_handle_count(&self) -> usize {
        self.event_receivers.len()
    }

    /// Get all received KlineUpdate events grouped by output handle ID
    ///
    /// # Returns
    ///
    /// Returns a clone of the HashMap where key is from_node_handle_id and value is Vec of events
    pub async fn get_received_kline_update_events(&self) -> HashMap<String, Vec<KlineUpdateEvent>> {
        let cache = self.received_kline_update_events.lock().await;
        cache.clone()
    }

    /// Get KlineUpdate events from a specific output handle
    ///
    /// # Parameters
    ///
    /// - `from_node_handle_id`: The output handle ID to query
    ///
    /// # Returns
    ///
    /// Returns a clone of events from the specified handle, or empty Vec if not found
    pub async fn get_kline_update_events_by_handle(&self, from_node_handle_id: &str) -> Vec<KlineUpdateEvent> {
        let cache = self.received_kline_update_events.lock().await;
        cache.get(from_node_handle_id).cloned().unwrap_or_default()
    }

    /// Get the count of received KlineUpdate events from a specific output handle
    ///
    /// # Parameters
    ///
    /// - `from_node_handle_id`: The output handle ID to query
    ///
    /// # Returns
    ///
    /// Returns the number of events from the specified handle
    pub async fn get_kline_update_count_by_handle(&self, from_node_handle_id: &str) -> usize {
        let cache = self.received_kline_update_events.lock().await;
        cache.get(from_node_handle_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Get the total count of all received KlineUpdate events
    ///
    /// # Returns
    ///
    /// Returns the total number of KlineUpdate events received from all handles
    pub async fn get_kline_update_count(&self) -> usize {
        let cache = self.received_kline_update_events.lock().await;
        cache.values().map(|v| v.len()).sum()
    }

    /// Get statistics about received events per handle
    ///
    /// # Returns
    ///
    /// Returns a HashMap where key is from_node_handle_id and value is event count
    pub async fn get_kline_update_stats(&self) -> HashMap<String, usize> {
        let cache = self.received_kline_update_events.lock().await;
        cache.iter().map(|(k, v)| (k.clone(), v.len())).collect()
    }

    /// Wait for a specific number of KlineUpdate events with timeout
    ///
    /// This method polls the cache every 100ms to check if the expected number
    /// of events has been received.
    ///
    /// # Parameters
    ///
    /// - `expected_count`: The expected number of KlineUpdate events (total across all handles)
    /// - `timeout`: Maximum duration to wait
    ///
    /// # Returns
    ///
    /// Returns true if expected_count events were received within timeout, false otherwise
    pub async fn wait_for_kline_updates(&self, expected_count: usize, timeout: Duration) -> bool {
        let start = tokio::time::Instant::now();
        let poll_interval = Duration::from_millis(100);

        while start.elapsed() < timeout {
            let count = self.get_kline_update_count().await;
            if count >= expected_count {
                tracing::info!(
                    "MockIndicatorNode [{}] received expected {} KlineUpdate events",
                    self.node_id,
                    expected_count
                );
                return true;
            }

            tokio::time::sleep(poll_interval).await;
        }

        let final_count = self.get_kline_update_count().await;
        tracing::warn!(
            "MockIndicatorNode [{}] timeout waiting for KlineUpdate events, expected: {}, received: {}",
            self.node_id,
            expected_count,
            final_count
        );
        false
    }

    /// Wait for a specific number of KlineUpdate events from a specific handle with timeout
    ///
    /// # Parameters
    ///
    /// - `from_node_handle_id`: The output handle ID to monitor
    /// - `expected_count`: The expected number of KlineUpdate events from this handle
    /// - `timeout`: Maximum duration to wait
    ///
    /// # Returns
    ///
    /// Returns true if expected_count events were received from the handle within timeout, false otherwise
    pub async fn wait_for_kline_updates_by_handle(
        &self,
        from_node_handle_id: &str,
        expected_count: usize,
        timeout: Duration,
    ) -> bool {
        let start = tokio::time::Instant::now();
        let poll_interval = Duration::from_millis(100);

        while start.elapsed() < timeout {
            let count = self.get_kline_update_count_by_handle(from_node_handle_id).await;
            if count >= expected_count {
                tracing::info!(
                    "MockIndicatorNode [{}] received expected {} KlineUpdate events from handle '{}'",
                    self.node_id,
                    expected_count,
                    from_node_handle_id
                );
                return true;
            }

            tokio::time::sleep(poll_interval).await;
        }

        let final_count = self.get_kline_update_count_by_handle(from_node_handle_id).await;
        tracing::warn!(
            "MockIndicatorNode [{}] timeout waiting for KlineUpdate events from handle '{}', expected: {}, received: {}",
            self.node_id,
            from_node_handle_id,
            expected_count,
            final_count
        );
        false
    }

    /// Clear all cached KlineUpdate events
    ///
    /// This is useful for resetting the cache between test runs
    pub async fn clear_kline_update_events(&self) {
        let mut cache = self.received_kline_update_events.lock().await;
        cache.clear();
        tracing::debug!("MockIndicatorNode [{}] cleared all cached KlineUpdate events", self.node_id);
    }
}

impl Drop for MockIndicatorNode {
    fn drop(&mut self) {
        for task in self.listen_tasks.drain(..) {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_indicator_node_creation() {
        let (tx, rx) = broadcast::channel(100);
        let mock_node = MockIndicatorNode::new("test_indicator_1".to_string(), vec![rx]);

        assert_eq!(mock_node.get_node_id(), "test_indicator_1");
        assert_eq!(mock_node.get_handle_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_indicator_node_multiple_handles() {
        let (tx1, rx1) = broadcast::channel(100);
        let (tx2, rx2) = broadcast::channel(100);
        let (tx3, rx3) = broadcast::channel(100);

        let mock_node = MockIndicatorNode::new(
            "test_indicator_multi".to_string(),
            vec![rx1, rx2, rx3]
        );

        assert_eq!(mock_node.get_handle_count(), 3);
    }

    #[tokio::test]
    async fn test_mock_indicator_node_start_stop() {
        let (tx, rx) = broadcast::channel(100);
        let mut mock_node = MockIndicatorNode::new("test_indicator_2".to_string(), vec![rx]);

        mock_node.start_listening();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        mock_node.stop_listening().await;
    }
}
