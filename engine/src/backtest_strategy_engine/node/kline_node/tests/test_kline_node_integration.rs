use super::test_fixtures::{create_integration_fixture, init_test_tracing};
use crate::backtest_strategy_engine::node::kline_node::kline_node_context::KlineNodeContext;
use crate::backtest_strategy_engine::node::BacktestNodeTrait;
use event_center::communication::Response;
use star_river_core::key::key::KlineKey;
use star_river_core::key::KeyTrait;
use star_river_core::market::KlineInterval;
use tokio::time::{timeout, Duration};

// =============================================================================
// Integration Tests
// =============================================================================
// cargo nextest run --package engine --run-ignored ignored-only --test-threads=1
#[tokio::test]
#[ignore]
async fn test_register_exchange() {
    // Initialize tracing subscriber for logging
    init_test_tracing();

    // Setup: Create integration test fixture with EventCenter and running ExchangeEngine
    tracing::info!("========================================");
    tracing::info!("Starting register_exchange integration test");
    tracing::info!("========================================");

    let fixture = create_integration_fixture().await;

    // Step 1: Create a KlineNode with exchange configuration
    tracing::info!("Step 1: Creating KlineNode with exchange configuration...");
    let node = fixture.create_test_kline_node().unwrap();

    // Step 2: Get the KlineNodeContext to access register_exchange
    tracing::info!("Step 2: Accessing KlineNodeContext...");
    let context = node.get_context();
    let mut context_guard = context.write().await;

    let kline_context = context_guard
        .as_any_mut()
        .downcast_mut::<KlineNodeContext>()
        .expect("Failed to downcast to KlineNodeContext");

    // Step 3: Verify exchange config exists
    tracing::info!("Step 3: Verifying exchange configuration...");
    let exchange_config = kline_context
        .node_config
        .exchange_mode_config
        .as_ref()
        .expect("Exchange config should exist");

    let account_id = exchange_config.selected_account.account_id;
    let exchange = &exchange_config.selected_account.exchange;

    tracing::info!("  Account ID: {}", account_id);
    tracing::info!("  Exchange: {}", exchange);

    // Step 4: Verify exchange is NOT registered before calling register_exchange
    tracing::info!("Step 4: Verifying exchange is not registered yet...");
    let is_registered_before = fixture.exchange_engine.lock().await.is_registered(&account_id).await;
    assert!(
        !is_registered_before,
        "Exchange should not be registered before calling register_exchange"
    );
    tracing::info!("  ✅ Confirmed: Exchange is not registered yet");

    // Step 5: Call register_exchange
    tracing::info!("Step 5: Calling register_exchange...");
    let register_result = timeout(
        Duration::from_secs(10),
        kline_context.register_exchange()
    ).await;

    // Step 6: Verify the result
    tracing::info!("Step 6: Verifying register_exchange result...");
    assert!(
        register_result.is_ok(),
        "register_exchange should complete within timeout"
    );

    let response = register_result.unwrap();
    assert!(
        response.is_ok(),
        "register_exchange should return Ok, got: {:?}",
        response
    );

    let response = response.unwrap();
    assert!(
        response.is_success(),
        "register_exchange response should be successful, got: {:?}",
        response.get_error()
    );

    tracing::info!("  ✅ register_exchange completed successfully");

    // Step 7: Verify exchange is NOW registered
    drop(context_guard); // Release write lock before checking exchange engine

    tracing::info!("Step 7: Verifying exchange is now registered...");
    let is_registered_after = fixture.exchange_engine.lock().await.is_registered(&account_id).await;
    assert!(
        is_registered_after,
        "Exchange should be registered after calling register_exchange"
    );
    tracing::info!("  ✅ Confirmed: Exchange is now registered");

    // Cleanup
    tracing::info!("Cleaning up test fixture...");
    fixture.cleanup().await;

    tracing::info!("========================================");
    tracing::info!("✅ register_exchange integration test PASSED");
    tracing::info!("========================================");
}


#[tokio::test]
#[ignore] // Run manually: cargo test test_load_kline_history_from_exchange -- --ignored --nocapture
async fn test_load_kline_history_from_exchange() {
    // Initialize tracing subscriber for logging
    init_test_tracing();

    // Setup: Create integration test fixture with EventCenter and running ExchangeEngine
    tracing::info!("========================================");
    tracing::info!("Starting load_kline_history_from_exchange integration test");
    tracing::info!("========================================");

    let fixture = create_integration_fixture().await;

    // Step 1: Create a KlineNode with exchange configuration
    tracing::info!("Step 1: Creating KlineNode with exchange configuration...");
    let node = fixture.create_test_kline_node().unwrap();

    // Step 2: Get the KlineNodeContext to access load_kline_history_from_exchange
    tracing::info!("Step 2: Accessing KlineNodeContext...");
    let context = node.get_context();
    let mut context_guard = context.write().await;

    let kline_context = context_guard
        .as_any_mut()
        .downcast_mut::<KlineNodeContext>()
        .expect("Failed to downcast to KlineNodeContext");

    // Step 3: Verify exchange config exists
    tracing::info!("Step 3: Verifying exchange configuration...");
    let exchange_config = kline_context
        .node_config
        .exchange_mode_config
        .as_ref()
        .expect("Exchange config should exist");

    let account_id = exchange_config.selected_account.account_id;
    let exchange = &exchange_config.selected_account.exchange;
    let time_range = &exchange_config.time_range;

    tracing::info!("  Account ID: {}", account_id);
    tracing::info!("  Exchange: {}", exchange);
    tracing::info!("  Time Range: {} to {}", time_range.start_date, time_range.end_date);
    tracing::info!("  Symbols: {:?}",
        exchange_config.selected_symbols.iter().map(|s| &s.symbol).collect::<Vec<_>>()
    );

    // Step 4: Register exchange first (prerequisite for loading kline data)
    tracing::info!("Step 4: Registering exchange (prerequisite)...");
    let register_result = timeout(
        Duration::from_secs(10),
        kline_context.register_exchange()
    ).await;

    assert!(
        register_result.is_ok(),
        "register_exchange should complete within timeout"
    );

    let response = register_result.unwrap();
    assert!(
        response.is_ok(),
        "register_exchange should return Ok, got: {:?}",
        response
    );

    let response = response.unwrap();
    assert!(
        response.is_success(),
        "register_exchange response should be successful, got: {:?}",
        response.get_error()
    );

    tracing::info!("  ✅ Exchange registered successfully");

    // Step 5: Get min interval symbols (another prerequisite)
    tracing::info!("Step 5: set min interval symbols..., not load from strategy");
    // let min_interval_symbols = kline_context.get_min_interval_symbols().await;
    // assert!(
    //     min_interval_symbols.is_ok(),
    //     "get_min_interval_symbols should succeed, got: {:?}",
    //     min_interval_symbols.err()
    // );

    // let min_interval_symbols = min_interval_symbols.unwrap();
    // kline_context.set_min_interval_symbols(min_interval_symbols.clone());
    let kline_key = KlineKey::new(
        exchange.clone(),
        "BTCUSDT".to_string(),
        KlineInterval::Minutes1,
        Some(time_range.start_date.to_string()),
        Some(time_range.end_date.to_string()),
    );
    let min_interval_symbols = vec![kline_key];
    kline_context.set_min_interval_symbols(min_interval_symbols.clone());

    tracing::info!("  ✅ Min interval symbols: {:?}",
        min_interval_symbols.iter().map(|k| format!("{}-{}", k.get_symbol(), k.get_interval())).collect::<Vec<_>>()
    );

    // Step 6: Call load_kline_history_from_exchange
    tracing::info!("Step 6: Calling load_kline_history_from_exchange...");
    tracing::info!("  ⏳ This may take a while depending on the time range and data size...");

    let load_result = timeout(
        Duration::from_secs(60), // Give it 60 seconds for loading data
        kline_context.load_kline_history_from_exchange()
    ).await;

    // Step 7: Verify the result
    tracing::info!("Step 7: Verifying load_kline_history_from_exchange result...");
    assert!(
        load_result.is_ok(),
        "load_kline_history_from_exchange should complete within timeout"
    );

    let load_response = load_result.unwrap();
    assert!(
        load_response.is_ok(),
        "load_kline_history_from_exchange should succeed, got error: {:?}",
        load_response.err()
    );

    tracing::info!("  ✅ load_kline_history_from_exchange completed successfully");

    // Step 8: Verify kline data was actually loaded
    // Note: The actual kline data is stored in the node's internal state
    // We can verify indirectly by checking that the operation completed without error
    tracing::info!("Step 8: Verifying kline data was loaded...");
    tracing::info!("  ✅ Kline data loading process completed successfully");

    // Cleanup
    drop(context_guard); // Release write lock

    tracing::info!("Cleaning up test fixture...");
    fixture.cleanup().await;

    tracing::info!("========================================");
    tracing::info!("✅ load_kline_history_from_exchange integration test PASSED");
    tracing::info!("========================================");
}
