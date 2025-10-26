#[cfg(test)]
mod test_kline_node_config;

#[cfg(test)]
mod test_check_kline_node_config;

#[cfg(test)]
mod test_kline_node_integration;

// =============================================================================
// Test Fixtures and Utilities (shared across all integration tests)
// =============================================================================

#[cfg(test)]
pub(crate) mod test_fixtures {
    use crate::backtest_strategy_engine::node::kline_node::KlineNode;
    use crate::exchange_engine::ExchangeEngine;
    use crate::market_engine::MarketEngine;
    use crate::Engine;
    use database::DatabaseManager;
    use entity::account_config;
    use event_center::communication::backtest_strategy::{
        NodeCommandReceiver, StrategyCommandSender, StrategyCommandReceiver,
    };
    use event_center::singleton::EventCenterSingleton;
    use heartbeat::Heartbeat;
    use sea_orm::{DatabaseConnection, EntityTrait, Set};
    use chrono::Utc;
    use serde_json::json;
    use star_river_core::custom_type::PlayIndex;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::time::Duration;

    /// Test fixture containing all required engines and infrastructure
    pub struct KlineNodeTestFixture {
        /// Database manager for managing test database
        pub database_manager: DatabaseManager,

        /// Database connection for persistence
        pub database: DatabaseConnection,

        /// Exchange engine for market data and trading operations
        pub exchange_engine: Arc<Mutex<ExchangeEngine>>,

        /// Market engine for kline data operations
        pub market_engine: Arc<Mutex<MarketEngine>>,

        /// Heartbeat for monitoring engine health
        pub heartbeat: Arc<Mutex<Heartbeat>>,

        /// Strategy command sender for node -> strategy communication
        pub strategy_command_sender: StrategyCommandSender,

        /// Strategy command receiver for receiving commands from nodes (used in tests to mock strategy)
        pub strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,

        /// Node command receiver for strategy -> node communication
        pub node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,

        /// Play index watch channel for coordinating playback
        pub play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        pub play_index_watch_tx: tokio::sync::watch::Sender<PlayIndex>,

        /// Background task handles for cleanup
        pub exchange_engine_task: Option<tokio::task::JoinHandle<()>>,
        pub market_engine_task: Option<tokio::task::JoinHandle<()>>,
        pub strategy_mock_task: Option<tokio::task::JoinHandle<()>>,
    }

    impl KlineNodeTestFixture {
        /// Create a new test fixture with all required infrastructure for integration testing
        ///
        /// # Returns
        ///
        /// A fully initialized test fixture ready for use in tests
        ///
        /// # Note
        ///
        /// This fixture:
        /// - Uses in-memory database for isolated testing
        /// - Initializes EventCenter singleton (only once per test process)
        /// - Creates ExchangeEngine and MarketEngine (not started yet)
        /// - Inserts test data into the database
        pub async fn new() -> Self {
            // 1. Setup in-memory database for testing
            let database_manager = DatabaseManager::new_in_memory()
                .await
                .expect("Failed to create in-memory database");
            let database = database_manager.get_conn();

            // 2. Insert test account data
            Self::insert_test_data(&database).await;

            // 3. Initialize EventCenter singleton
            // If already initialized by another test, the init will fail but that's okay
            match EventCenterSingleton::init().await {
                Ok(_) => {
                    tracing::info!("EventCenter initialized for integration tests");
                }
                Err(_) => {
                    tracing::info!("EventCenter already initialized, reusing existing instance");
                }
            }

            // 4. Create heartbeat (with 1 second interval)
            let heartbeat = Arc::new(Mutex::new(Heartbeat::new(1000)));

            // 5. Create exchange engine
            let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(database.clone())));

            // 6. Create market engine
            let market_engine = Arc::new(Mutex::new(MarketEngine::new(exchange_engine.clone())));

            // 7. Create communication channels
            let (strategy_command_sender, strategy_command_receiver) =
                tokio::sync::mpsc::channel(100);

            let strategy_command_receiver = Arc::new(Mutex::new(strategy_command_receiver));

            let (_node_command_sender, node_command_receiver) = tokio::sync::mpsc::channel(100);

            let node_command_receiver = Arc::new(Mutex::new(node_command_receiver));

            // 8. Create play index watch channel
            let (play_index_watch_tx, play_index_watch_rx) = tokio::sync::watch::channel(0);

            Self {
                database_manager,
                database,
                exchange_engine,
                market_engine,
                heartbeat,
                strategy_command_sender,
                strategy_command_receiver,
                node_command_receiver,
                play_index_watch_rx,
                play_index_watch_tx,
                exchange_engine_task: None,
                market_engine_task: None,
                strategy_mock_task: None,
            }
        }

        /// Insert test data into the database
        async fn insert_test_data(database: &DatabaseConnection) {
            // Insert test account with id=2 (binance)
            let test_account = account_config::ActiveModel {
                id: Set(2),
                account_name: Set("Test Binance Account".to_string()),
                exchange: Set("binance".to_string()),
                is_available: Set(true),
                is_delete: Set(false),
                sort_index: Set(1),
                account_config: Set(json!({
                    "api_key": "test_api_key",
                    "api_secret": "test_api_secret"
                })),
                create_time: Set(Utc::now()),
                update_time: Set(Utc::now()),
            };

            account_config::Entity::insert(test_account)
                .exec(database)
                .await
                .expect("Failed to insert test account data");

            tracing::debug!("Test account data inserted successfully");
        }

        /// Create a test KlineNode with valid configuration
        ///
        /// # Returns
        ///
        /// A KlineNode instance configured for testing
        pub fn create_test_kline_node(&self) -> Result<KlineNode, Box<dyn std::error::Error>> {
            let config = self.create_valid_node_config();

            let node = KlineNode::new(
                config,
                self.strategy_command_sender.clone(),
                self.node_command_receiver.clone(),
                self.play_index_watch_rx.clone(),
            )?;

            Ok(node)
        }

        /// Create a valid node configuration for testing
        fn create_valid_node_config(&self) -> serde_json::Value {
            json!({
                "id": "test_kline_node_1",
                "data": {
                    "nodeName": "Test Kline Node",
                    "strategyId": 1,
                    "backtestConfig": {
                        "dataSource": "exchange",
                        "exchangeModeConfig": {
                            "selectedAccount": {
                                "accountName": "Test Account",
                                "availableBalance": 10000.0,
                                "exchange": "binance",
                                "id": 2
                            },
                            "selectedSymbols": [
                                {
                                    "configId": 1,
                                    "interval": "1m",
                                    "outputHandleId": "test_kline_node_1_output_1",
                                    "symbol": "BTCUSDT"
                                }
                            ],
                            "timeRange": {
                                "startDate": "2025-01-01 00:00:00 +08:00",
                                "endDate": "2025-01-02 00:00:00 +08:00"
                            }
                        }
                    }
                }
            })
        }

        /// Start the exchange engine and market engine in background tasks
        ///
        /// # Returns
        ///
        /// Returns a new fixture with both engines running
        pub async fn start_engines(mut self) -> Self {
            // Start exchange engine in background task
            let exchange_engine = self.exchange_engine.clone();
            let exchange_task = tokio::spawn(async move {
                let engine = exchange_engine.lock().await;
                tracing::info!("Starting ExchangeEngine...");
                engine.start().await;
            });
            self.exchange_engine_task = Some(exchange_task);

            // Start market engine in background task
            let market_engine = self.market_engine.clone();
            let market_task = tokio::spawn(async move {
                let engine = market_engine.lock().await;
                tracing::info!("Starting MarketEngine...");
                engine.start().await;
            });
            self.market_engine_task = Some(market_task);

            // Start strategy mock task to handle commands from nodes
            let strategy_command_receiver = self.strategy_command_receiver.clone();
            let strategy_mock_task = tokio::spawn(async move {
                tracing::info!("Starting strategy mock task...");
                loop {
                    let mut receiver = strategy_command_receiver.lock().await;
                    if let Some(command) = receiver.recv().await {
                        use event_center::communication::backtest_strategy::BacktestStrategyCommand;
                        match command {
                            BacktestStrategyCommand::InitKlineData(cmd) => {
                                tracing::debug!("Strategy mock received InitKlineData command");
                                let response =
                                    event_center::communication::backtest_strategy::InitKlineDataResponse::success(
                                        Some(
                                            event_center::communication::backtest_strategy::strategy_command::InitKlineDataRespPayload,
                                        ),
                                    );
                                let _ = cmd.command_base.responder.send(response);
                            }
                            BacktestStrategyCommand::AppendKlineData(cmd) => {
                                tracing::debug!("Strategy mock received AppendKlineData command");
                                let response =
                                    event_center::communication::backtest_strategy::AppendKlineDataResponse::success(
                                        Some(
                                            event_center::communication::backtest_strategy::strategy_command::AppendKlineDataRespPayload,
                                        ),
                                    );
                                let _ = cmd.command_base.responder.send(response);
                            }
                            _ => {
                                tracing::warn!(
                                    "Strategy mock received unhandled command: {:?}",
                                    command
                                );
                            }
                        }
                    } else {
                        break;
                    }
                }
                tracing::info!("Strategy mock task stopped");
            });
            self.strategy_mock_task = Some(strategy_mock_task);

            // Give the engines some time to start listening for commands
            tokio::time::sleep(Duration::from_millis(100)).await;

            tracing::info!("ExchangeEngine, MarketEngine, and Strategy mock started in background");

            self
        }

        /// Cleanup test resources
        pub async fn cleanup(self) {
            tracing::debug!("Cleaning up test fixture...");

            // Abort background tasks
            if let Some(task) = self.exchange_engine_task {
                task.abort();
                tracing::debug!("Exchange engine task aborted");
            }

            if let Some(task) = self.market_engine_task {
                task.abort();
                tracing::debug!("Market engine task aborted");
            }

            if let Some(task) = self.strategy_mock_task {
                task.abort();
                tracing::debug!("Strategy mock task aborted");
            }

            // DatabaseManager will be dropped automatically
            // Exchange engine and Market engine will be dropped automatically
            // All Arc/Mutex will be dropped when no longer referenced

            tracing::debug!("Test fixture cleanup complete");
        }
    }

    // =============================================================================
    // Helper Functions
    // =============================================================================

    /// Initialize tracing subscriber for test logging
    ///
    /// This should be called at the beginning of each integration test.
    /// It's safe to call multiple times - subsequent calls will be ignored.
    ///
    /// Note: Removes `with_test_writer()` to enable real-time log output.
    /// This allows logs to be printed immediately as tests run, rather than
    /// being buffered and printed at the end.
    pub fn init_test_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            // Removed .with_test_writer() to enable real-time output
            .try_init();
    }

    /// Create a fixture with running engines for full integration tests
    pub async fn create_integration_fixture() -> KlineNodeTestFixture {
        KlineNodeTestFixture::new().await.start_engines().await
    }
}
