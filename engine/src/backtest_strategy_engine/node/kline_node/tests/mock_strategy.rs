use chrono::{TimeZone, Utc};
use event_center::communication::backtest_strategy::{
    AddNodeCycleTrackerResponse, BacktestStrategyCommand, GetKlineDataRespPayload, GetKlineDataResponse, GetMinIntervalSymbolsRespPayload, GetMinIntervalSymbolsResponse, StrategyCommandReceiver, UpdateKlineDataResponse
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use star_river_core::key::key::KlineKey;
use star_river_core::market::{Exchange, Kline, KlineInterval};
use strategy_core::strategy::TimeRange;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

/// Mock strategy for testing KlineNode
///
/// This mock simulates a strategy that responds to commands from nodes.
/// It handles InitKlineData, AppendKlineData, and GetKlineData commands.
/// It also subscribes to node's strategy_output_handle to receive node events.
pub struct MockStrategy {
    /// Command receiver for receiving commands from nodes
    command_receiver: Arc<Mutex<StrategyCommandReceiver>>,

    /// Pre-generated mock kline data
    mock_klines: Vec<Kline>,

    /// Background task handles for cleanup
    command_task_handle: Option<tokio::task::JoinHandle<()>>,
    event_listener_task_handle: Option<tokio::task::JoinHandle<()>>,

    /// Kline node configuration
    kline_node_config: serde_json::Value,

    /// Receivers for node strategy output handles
    node_event_receivers: Vec<broadcast::Receiver<BacktestNodeEvent>>,
}

impl MockStrategy {
    /// Create a new mock strategy
    ///
    /// # Parameters
    ///
    /// - `command_receiver`: Receiver for strategy commands from nodes
    /// - `kline_count`: Number of mock klines to generate
    /// - `start_timestamp`: Starting timestamp (Unix timestamp in milliseconds)
    /// - `interval_ms`: Interval between klines in milliseconds
    /// - `kline_node_config`: Kline node configuration JSON
    ///
    /// # Returns
    ///
    /// Returns a MockStrategy instance
    pub fn new(
        command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        kline_count: usize,
        start_timestamp: i64,
        interval_ms: i64,
        kline_node_config: serde_json::Value,
    ) -> Self {
        let mock_klines = generate_mock_klines(kline_count, start_timestamp, interval_ms);
        tracing::debug!("MockStrategy generated {} mock klines", mock_klines.len());

        Self {
            command_receiver,
            mock_klines,
            command_task_handle: None,
            event_listener_task_handle: None,
            kline_node_config,
            node_event_receivers: Vec::new(),
        }
    }

    /// Add a node's strategy output handle receiver
    ///
    /// This allows the mock strategy to receive events from the node's strategy_output_handle
    pub fn add_node_event_receiver(&mut self, receiver: broadcast::Receiver<BacktestNodeEvent>) {
        self.node_event_receivers.push(receiver);
        tracing::debug!(
            "MockStrategy added node event receiver, total: {}",
            self.node_event_receivers.len()
        );
    }

    /// Create a mock strategy with default parameters
    ///
    /// Default: 1500 klines starting from 2025-01-01 00:00:00 UTC, 1-minute intervals
    pub fn new_default(command_receiver: Arc<Mutex<StrategyCommandReceiver>>, kline_node_config: serde_json::Value) -> Self {
        let start_timestamp = Utc
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap()
            .timestamp_millis();

        Self::new(command_receiver, 1500, start_timestamp, 60_000, kline_node_config)
    }

    /// Start the mock strategy tasks in background
    ///
    /// This spawns two background tasks:
    /// 1. Command handler - listens for commands from nodes and responds
    /// 2. Event listener - listens for events from node's strategy_output_handle
    pub fn start(&mut self) {
        // Start command handler task
        self.start_command_handler();

        // Start event listener task (if there are any receivers)
        if !self.node_event_receivers.is_empty() {
            self.start_event_listener();
        }

        tracing::info!("MockStrategy started with {} event receivers", self.node_event_receivers.len());
    }

    /// Start the command handler task
    fn start_command_handler(&mut self) {
        let command_receiver = self.command_receiver.clone();
        let mock_klines = self.mock_klines.clone();
        let kline_node_config = self.kline_node_config.clone();

        let task = tokio::spawn(async move {
            tracing::info!("MockStrategy task started");

            loop {
                let mut receiver = command_receiver.lock().await;
                if let Some(command) = receiver.recv().await {
                    drop(receiver); // Release lock before processing

                    match command {
                        BacktestStrategyCommand::InitKlineData(cmd) => {
                            tracing::debug!("MockStrategy received InitKlineData command");
                            let response =
                                event_center::communication::backtest_strategy::InitKlineDataResponse::success(
                                    Some(
                                        event_center::communication::backtest_strategy::strategy_command::InitKlineDataRespPayload,
                                    ),
                                );
                            let _ = cmd.command_base.responder.send(response);
                        }
                        BacktestStrategyCommand::AppendKlineData(cmd) => {
                            tracing::debug!("MockStrategy received AppendKlineData command");
                            let response =
                                event_center::communication::backtest_strategy::AppendKlineDataResponse::success(
                                    Some(
                                        event_center::communication::backtest_strategy::strategy_command::AppendKlineDataRespPayload,
                                    ),
                                );
                            let _ = cmd.command_base.responder.send(response);
                        }
                        BacktestStrategyCommand::GetKlineData(cmd) => {
                            tracing::debug!("MockStrategy received GetKlineData command");

                            let kline_slice = Self::get_kline_slice(
                                &mock_klines,
                                cmd.play_index.map(|p| p as u32),
                                cmd.limit.map(|l| l as usize),
                            );

                            tracing::debug!(
                                "MockStrategy returning {} klines (play_index: {:?}, limit: {:?})",
                                kline_slice.len(),
                                cmd.play_index,
                                cmd.limit
                            );

                            let resp_payload = GetKlineDataRespPayload::new(kline_slice);
                            let response = GetKlineDataResponse::success(Some(resp_payload));
                            let _ = cmd.command_base.responder.send(response);
                        }
                        BacktestStrategyCommand::GetMinIntervalSymbols(cmd) => {
                            tracing::debug!("MockStrategy received GetMinIntervalSymbols command");

                            // Parse kline node config to extract min interval symbols
                            let min_interval_keys = Self::extract_min_interval_symbols(&kline_node_config);

                            tracing::debug!(
                                "MockStrategy returning min interval symbols: {:?}",
                                min_interval_keys
                            );

                            let resp_payload = GetMinIntervalSymbolsRespPayload::new(min_interval_keys);
                            let response = GetMinIntervalSymbolsResponse::success(Some(resp_payload));
                            let _ = cmd.command_base.responder.send(response);
                        }
                        BacktestStrategyCommand::AddNodeCycleTracker(cmd) => {
                            tracing::debug!("MockStrategy received AddNodeCycleTracker command");
                            let response = AddNodeCycleTrackerResponse::success(None);
                            let _ = cmd.command_base.responder.send(response);
                        }
                        BacktestStrategyCommand::UpdateKlineData(cmd) => {
                            tracing::debug!("MockStrategy received UpdateKlineData command");
                            let response = UpdateKlineDataResponse::success(None);
                            let _ = cmd.command_base.responder.send(response);
                        }
                        _ => {
                            tracing::warn!("MockStrategy received unhandled command: {:?}", command);
                        }
                    }
                } else {
                    break;
                }
            }

            tracing::info!("MockStrategy command handler task stopped");
        });

        self.command_task_handle = Some(task);
        tracing::debug!("MockStrategy command handler task spawned");
    }

    /// Start the event listener task
    fn start_event_listener(&mut self) {
        // Take ownership of receivers to move into the async task
        let receivers: Vec<_> = self
            .node_event_receivers
            .drain(..)
            .map(|rx| Arc::new(Mutex::new(rx)))
            .collect();

        let receiver_count = receivers.len();

        let task = tokio::spawn(async move {
            tracing::info!(
                "MockStrategy event listener started for {} receivers",
                receiver_count
            );

            // Create tasks for each receiver
            let mut tasks = Vec::new();
            for (index, receiver) in receivers.iter().enumerate() {
                let receiver = receiver.clone();
                let task = tokio::spawn(async move {
                    loop {
                        let mut rx = receiver.lock().await;
                        match rx.recv().await {
                            Ok(event) => {
                                tracing::info!(
                                    "MockStrategy receiver #{} received node event: {:?}",
                                    index,
                                    event
                                );
                            }
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!(
                                    "MockStrategy receiver #{} lagged, skipped {} events",
                                    index,
                                    skipped
                                );
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                tracing::info!(
                                    "MockStrategy receiver #{} channel closed",
                                    index
                                );
                                break;
                            }
                        }
                    }
                });
                tasks.push(task);
            }

            // Wait for all tasks to complete
            for task in tasks {
                let _ = task.await;
            }

            tracing::info!("MockStrategy event listener task stopped");
        });

        self.event_listener_task_handle = Some(task);
        tracing::debug!("MockStrategy event listener task spawned");
    }

    /// Stop all mock strategy tasks
    pub async fn stop(&mut self) {
        if let Some(task) = self.command_task_handle.take() {
            task.abort();
            tracing::info!("MockStrategy command handler task aborted");
        }

        if let Some(task) = self.event_listener_task_handle.take() {
            task.abort();
            tracing::info!("MockStrategy event listener task aborted");
        }
    }

    /// Get a slice of kline data based on play_index and limit
    ///
    /// This implements the same logic as the real strategy's get_kline_data method
    fn get_kline_slice(
        klines: &[Kline],
        play_index: Option<u32>,
        limit: Option<usize>,
    ) -> Vec<Kline> {
        let kline_data_length = klines.len();

        match (play_index, limit) {
            // Has index, has limit - get 'limit' elements backwards from index
            (Some(play_index), Some(limit)) => {
                let play_index = play_index as usize;

                // If index is out of range, return empty
                if play_index >= kline_data_length {
                    tracing::warn!(
                        "Play index {} out of range (data length: {})",
                        play_index,
                        kline_data_length
                    );
                    Vec::new()
                } else {
                    let end = play_index + 1;
                    let start = if limit >= end { 0 } else { end - limit };
                    klines[start..end].to_vec()
                }
            }
            // Has index, no limit - get all elements from start to index
            (Some(play_index), None) => {
                let play_index = play_index as usize;

                // If index is out of range, return empty
                if play_index >= kline_data_length {
                    tracing::warn!(
                        "Play index {} out of range (data length: {})",
                        play_index,
                        kline_data_length
                    );
                    Vec::new()
                } else {
                    let end = play_index + 1;
                    klines[0..end].to_vec()
                }
            }
            // No index, has limit - get last 'limit' klines
            (None, Some(limit)) => {
                if limit >= kline_data_length {
                    klines.to_vec()
                } else {
                    let start = kline_data_length.saturating_sub(limit);
                    klines[start..].to_vec()
                }
            }
            // No index, no limit - return all data
            (None, None) => klines.to_vec(),
        }
    }

    /// Get the number of mock klines
    pub fn kline_count(&self) -> usize {
        self.mock_klines.len()
    }

    /// Extract minimum interval symbols from kline node configuration
    ///
    /// For each unique symbol in the configuration, this method finds the smallest interval
    /// and creates a KlineKey with that interval.
    ///
    /// # Parameters
    ///
    /// - `config`: The kline node configuration JSON
    ///
    /// # Returns
    ///
    /// A vector of KlineKey representing the minimum interval for each symbol
    fn extract_min_interval_symbols(config: &serde_json::Value) -> Vec<KlineKey> {
        let mut symbol_min_intervals: HashMap<String, KlineInterval> = HashMap::new();
        let mut exchange: Option<Exchange> = None;
        let mut time_range: Option<TimeRange> = None;

        // Parse the configuration structure
        if let Some(data) = config.get("data") {
            if let Some(backtest_config) = data.get("backtestConfig") {
                if let Some(exchange_config) = backtest_config.get("exchangeModeConfig") {
                    // Extract exchange from selectedAccount
                    if let Some(account) = exchange_config.get("selectedAccount") {
                        if let Some(exchange_str) = account.get("exchange").and_then(|e| e.as_str())
                        {
                            exchange = exchange_str.parse::<Exchange>().ok();
                        }
                    }

                    // Extract time range using TimeRange::new()
                    if let Some(time_range_obj) = exchange_config.get("timeRange") {
                        if let (Some(start), Some(end)) = (
                            time_range_obj.get("startDate").and_then(|s| s.as_str()),
                            time_range_obj.get("endDate").and_then(|e| e.as_str()),
                        ) {
                            time_range = Some(TimeRange::new(start.to_string(), end.to_string()));
                        }
                    }

                    // Extract symbols and intervals
                    if let Some(selected_symbols) = exchange_config.get("selectedSymbols") {
                        if let Some(symbols_array) = selected_symbols.as_array() {
                            for symbol_config in symbols_array {
                                if let (Some(symbol), Some(interval_str)) = (
                                    symbol_config.get("symbol").and_then(|s| s.as_str()),
                                    symbol_config.get("interval").and_then(|i| i.as_str()),
                                ) {
                                    if let Ok(interval) = interval_str.parse::<KlineInterval>() {
                                        // Update if this is the first interval for this symbol
                                        // or if this interval is smaller than the current minimum
                                        symbol_min_intervals
                                            .entry(symbol.to_string())
                                            .and_modify(|current_min| {
                                                if interval < *current_min {
                                                    *current_min = interval.clone();
                                                }
                                            })
                                            .or_insert(interval);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Convert the HashMap to Vec<KlineKey>
        let exchange = exchange.unwrap_or(Exchange::Binance);
        symbol_min_intervals
            .into_iter()
            .map(|(symbol, interval)| {
                KlineKey::new(
                    exchange.clone(),
                    symbol,
                    interval,
                    time_range.as_ref().map(|tr| tr.start_date.to_string()),
                    time_range.as_ref().map(|tr| tr.end_date.to_string()),
                )
            })
            .collect()
    }
}

impl Drop for MockStrategy {
    fn drop(&mut self) {
        if let Some(task) = self.command_task_handle.take() {
            task.abort();
        }
        if let Some(task) = self.event_listener_task_handle.take() {
            task.abort();
        }
    }
}

/// Generate mock kline data for testing
///
/// # Parameters
///
/// - `count`: Number of klines to generate
/// - `start_time`: Starting timestamp (Unix timestamp in milliseconds)
/// - `interval_ms`: Interval between klines in milliseconds
///
/// # Returns
///
/// A vector of mock Kline data with simulated price movements
fn generate_mock_klines(count: usize, start_time: i64, interval_ms: i64) -> Vec<Kline> {
    let mut klines = Vec::with_capacity(count);
    let mut current_time = start_time;
    let mut base_price = 50000.0;

    for i in 0..count {
        // Simulate price movement using sine wave
        let price_change = (i as f64 * 0.1).sin() * 100.0;
        base_price += price_change;

        let open = base_price;
        let close = base_price + (i as f64 * 0.5);
        let high = base_price.max(close) + 50.0;
        let low = base_price.min(close) - 50.0;
        let volume = 100.0 + (i as f64 * 10.0);

        let datetime = Utc.timestamp_millis_opt(current_time).unwrap();

        let kline = Kline::new(datetime, open, high, low, close, volume);
        klines.push(kline);

        current_time += interval_ms;
    }

    klines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_klines() {
        let start_time = Utc
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .unwrap()
            .timestamp_millis();
        let klines = generate_mock_klines(100, start_time, 60_000);

        assert_eq!(klines.len(), 100);
        assert_eq!(klines[0].datetime.timestamp_millis(), start_time);
        assert_eq!(
            klines[1].datetime.timestamp_millis(),
            start_time + 60_000
        );
    }

    #[test]
    fn test_get_kline_slice_with_index_and_limit() {
        let klines: Vec<Kline> = (0..100)
            .map(|i| {
                let datetime = Utc.timestamp_millis_opt(i * 60_000).unwrap();
                Kline::new(datetime, 100.0, 110.0, 90.0, 105.0, 1000.0)
            })
            .collect();

        // Get 10 klines up to index 50
        let slice = MockStrategy::get_kline_slice(&klines, Some(50), Some(10));
        assert_eq!(slice.len(), 10);
        assert_eq!(slice[0].datetime.timestamp_millis(), 41 * 60_000);
        assert_eq!(slice[9].datetime.timestamp_millis(), 50 * 60_000);
    }

    #[test]
    fn test_get_kline_slice_with_index_only() {
        let klines: Vec<Kline> = (0..100)
            .map(|i| {
                let datetime = Utc.timestamp_millis_opt(i * 60_000).unwrap();
                Kline::new(datetime, 100.0, 110.0, 90.0, 105.0, 1000.0)
            })
            .collect();

        // Get all klines from start to index 10
        let slice = MockStrategy::get_kline_slice(&klines, Some(10), None);
        assert_eq!(slice.len(), 11); // 0 to 10 inclusive
        assert_eq!(slice[0].datetime.timestamp_millis(), 0);
        assert_eq!(slice[10].datetime.timestamp_millis(), 10 * 60_000);
    }

    #[test]
    fn test_get_kline_slice_with_limit_only() {
        let klines: Vec<Kline> = (0..100)
            .map(|i| {
                let datetime = Utc.timestamp_millis_opt(i * 60_000).unwrap();
                Kline::new(datetime, 100.0, 110.0, 90.0, 105.0, 1000.0)
            })
            .collect();

        // Get last 10 klines
        let slice = MockStrategy::get_kline_slice(&klines, None, Some(10));
        assert_eq!(slice.len(), 10);
        assert_eq!(slice[0].datetime.timestamp_millis(), 90 * 60_000);
        assert_eq!(slice[9].datetime.timestamp_millis(), 99 * 60_000);
    }

    #[test]
    fn test_get_kline_slice_no_params() {
        let klines: Vec<Kline> = (0..100)
            .map(|i| {
                let datetime = Utc.timestamp_millis_opt(i * 60_000).unwrap();
                Kline::new(datetime, 100.0, 110.0, 90.0, 105.0, 1000.0)
            })
            .collect();

        // Get all klines
        let slice = MockStrategy::get_kline_slice(&klines, None, None);
        assert_eq!(slice.len(), 100);
    }

    #[test]
    fn test_get_kline_slice_out_of_range() {
        let klines: Vec<Kline> = (0..100)
            .map(|i| {
                let datetime = Utc.timestamp_millis_opt(i * 60_000).unwrap();
                Kline::new(datetime, 100.0, 110.0, 90.0, 105.0, 1000.0)
            })
            .collect();

        // Index out of range
        let slice = MockStrategy::get_kline_slice(&klines, Some(150), Some(10));
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_extract_min_interval_symbols_single_symbol() {
        use serde_json::json;

        let config = json!({
            "id": "test_kline_node_1",
            "data": {
                "backtestConfig": {
                    "exchangeModeConfig": {
                        "selectedAccount": {
                            "exchange": "binance"
                        },
                        "selectedSymbols": [
                            {
                                "symbol": "BTCUSDT",
                                "interval": "1m"
                            }
                        ],
                        "timeRange": {
                            "startDate": "2025-01-01 00:00:00 +08:00",
                            "endDate": "2025-01-02 00:00:00 +08:00"
                        }
                    }
                }
            }
        });

        let keys = MockStrategy::extract_min_interval_symbols(&config);
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].symbol, "BTCUSDT");
        assert_eq!(keys[0].interval, KlineInterval::Minutes1);
    }

    #[test]
    fn test_extract_min_interval_symbols_multiple_symbols() {
        use serde_json::json;

        let config = json!({
            "id": "test_kline_node_2",
            "data": {
                "backtestConfig": {
                    "exchangeModeConfig": {
                        "selectedAccount": {
                            "exchange": "binance"
                        },
                        "selectedSymbols": [
                            {
                                "symbol": "BTCUSDT",
                                "interval": "1m"
                            },
                            {
                                "symbol": "ETHUSDT",
                                "interval": "5m"
                            }
                        ],
                        "timeRange": {
                            "startDate": "2025-01-01 00:00:00 +08:00",
                            "endDate": "2025-01-02 00:00:00 +08:00"
                        }
                    }
                }
            }
        });

        let keys = MockStrategy::extract_min_interval_symbols(&config);
        assert_eq!(keys.len(), 2);

        // Find BTCUSDT and ETHUSDT
        let btc_key = keys.iter().find(|k| k.symbol == "BTCUSDT");
        let eth_key = keys.iter().find(|k| k.symbol == "ETHUSDT");

        assert!(btc_key.is_some());
        assert!(eth_key.is_some());
        assert_eq!(btc_key.unwrap().interval, KlineInterval::Minutes1);
        assert_eq!(eth_key.unwrap().interval, KlineInterval::Minutes5);
    }

    #[test]
    fn test_extract_min_interval_symbols_same_symbol_multiple_intervals() {
        use serde_json::json;

        // Test case: same symbol with different intervals - should return the minimum
        let config = json!({
            "id": "test_kline_node_3",
            "data": {
                "backtestConfig": {
                    "exchangeModeConfig": {
                        "selectedAccount": {
                            "exchange": "binance"
                        },
                        "selectedSymbols": [
                            {
                                "symbol": "BTCUSDT",
                                "interval": "5m"
                            },
                            {
                                "symbol": "BTCUSDT",
                                "interval": "1m"
                            },
                            {
                                "symbol": "BTCUSDT",
                                "interval": "15m"
                            }
                        ],
                        "timeRange": {
                            "startDate": "2025-01-01 00:00:00 +08:00",
                            "endDate": "2025-01-02 00:00:00 +08:00"
                        }
                    }
                }
            }
        });

        let keys = MockStrategy::extract_min_interval_symbols(&config);
        assert_eq!(keys.len(), 1, "Should only have one key for BTCUSDT");
        assert_eq!(keys[0].symbol, "BTCUSDT");
        assert_eq!(
            keys[0].interval,
            KlineInterval::Minutes1,
            "Should return the minimum interval (1m)"
        );
    }
}
