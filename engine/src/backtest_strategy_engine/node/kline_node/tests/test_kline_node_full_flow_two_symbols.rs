use super::test_fixtures::{create_integration_fixture, init_test_tracing};
use crate::backtest_strategy_engine::node::{node_types::NodeOutputHandle, BacktestNodeTrait};
use tokio::time::{timeout, Duration};

/// Test full flow: start node sends event -> kline node receives -> fetch data -> send data
///
/// This test demonstrates how to use MockStartNode to simulate the complete data flow
#[tokio::test]
#[ignore] 
// cargo nextest run --package engine --run-ignored ignored-only --test-threads=1
async fn test_full_kline_flow_two_symbols_with_different_interval() {
    // Initialize tracing subscriber for logging
    init_test_tracing();

    const SEND_COUNT: usize = 5;

    tracing::info!("========================================");
    tracing::info!("Starting full kline flow integration test");
    tracing::info!("========================================");

    let mut fixture = create_integration_fixture().await;
    let kline_node_config = fixture.create_two_symbols_with_different_interval_config();

    // Step 1: Create MockStartNode that will send 5 events
    tracing::info!("Step 1: Creating MockStartNode with 5 events...");
    let (mock_start_node, input_handle) = fixture.create_mock_start_node(SEND_COUNT);
    tracing::info!("  ✅ MockStartNode [{}] created", mock_start_node.get_node_id());

    // Step 2: Create KlineNode and connect to MockStartNode
    tracing::info!("Step 2: Creating KlineNode...");
    let mut kline_node = fixture.create_test_kline_node(kline_node_config.clone()).unwrap();
    kline_node.set_output_handle().await;
    kline_node.add_node_event_receiver(input_handle).await; // add start node output handle to kline node
    tracing::info!("  ✅ KlineNode created and connected to MockStartNode");

    // Step 3: Create MockIndicatorNode to subscribe to all KlineNode's output handles
    tracing::info!("Step 3: Creating MockIndicatorNode to receive kline data...");
    let strategy_output_handle_id = kline_node.get_strategy_output_handle().await.output_handle_id();
    let kline_node_output_handles = kline_node
        .get_all_output_handles()
        .await
        .iter()
        .filter(|handle| handle.output_handle_id != strategy_output_handle_id)
        .cloned()
        .collect::<Vec<NodeOutputHandle>>();
    tracing::info!("  Found {} output handles from KlineNode", kline_node_output_handles.len());

    let mut mock_indicator = fixture.create_mock_indicator_node(
        "test_indicator_1".to_string(),
        &kline_node_output_handles
    );
    mock_indicator.start_listening();
    tracing::info!(
        "  ✅ MockIndicatorNode created and started listening on {} handles",
        mock_indicator.get_handle_count()
    );

    // Step 4: Create and start MockStrategy, then subscribe to KlineNode's strategy output handle
    tracing::info!("Step 4: Creating MockStrategy and subscribing to node events...");
    fixture.create_mock_strategy(Some(kline_node_config));
    let mut strategy_output_handle = kline_node.get_strategy_output_handle().await;
    // kline_node.add_output_handle_connect_count(&strategy_output_handle.output_handle_id).await;
    fixture.subscribe_to_node_events(strategy_output_handle.subscribe());
    tracing::info!("  ✅ MockStrategy created and subscribed to KlineNode events");

    let init_result = timeout(
        Duration::from_secs(60),
        kline_node.init()
    ).await;

    assert!(init_result.is_ok(), "Init timeout");
    assert!(init_result.unwrap().is_ok(), "Failed to initialize KlineNode");
    tracing::info!("  ✅ KlineNode initialized (exchange registered, symbols loaded, history loaded)");

    // Step 6: Send events from MockStartNode
    tracing::info!("Step 6: MockStartNode sending events...");
    let sent_count = mock_start_node.send_events().await;
    assert_eq!(sent_count, SEND_COUNT, "Should send {} events", SEND_COUNT);
    tracing::info!("  ✅ MockStartNode sent {} events", sent_count);

    // Step 7: Verify MockIndicatorNode received events
    tracing::info!("Step 7: Verifying event processing...");

    // Wait for all events to be received (with timeout)
    let wait_timeout = Duration::from_secs(15);
    let received = mock_indicator.wait_for_kline_updates(SEND_COUNT, wait_timeout).await;
    assert!(
        received,
        "MockIndicatorNode should receive {} KlineUpdate events within timeout",
        SEND_COUNT
    );
    tracing::info!("  ✅ MockIndicatorNode received expected total event count: {}", SEND_COUNT);

    // Verify statistics per output handle
    let stats = mock_indicator.get_kline_update_stats().await;
    tracing::info!("  Event statistics by output handle:");
    for (handle_id, count) in &stats {
        tracing::info!("    - Handle '{}': {} events", handle_id, count);
    }

    // Verify each output handle received exactly SEND_COUNT events
    let all_events = mock_indicator.get_received_kline_update_events().await;
    assert!(!all_events.is_empty(), "Should have received events from at least one handle");

    for (handle_id, events) in &all_events {
        assert_eq!(
            events.len(),
            SEND_COUNT,
            "Handle '{}' should receive exactly {} events, but got {}",
            handle_id,
            SEND_COUNT,
            events.len()
        );
        tracing::info!("  ✅ Handle '{}' received expected {} events", handle_id, SEND_COUNT);
    }

    tracing::info!("  ✅ All output handles received correct event counts");

    // Cleanup
    tracing::info!("Cleaning up test fixture...");
    mock_indicator.stop_listening().await;
    fixture.cleanup().await;

    tracing::info!("========================================");
    tracing::info!("✅ full kline flow integration test COMPLETED");
    tracing::info!("Note: Some connection mechanisms still need implementation");
    tracing::info!("========================================");
}
