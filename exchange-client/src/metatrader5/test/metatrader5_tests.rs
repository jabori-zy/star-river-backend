#[cfg(test)]
mod tests {
    use crate::metatrader5::{MetaTrader5, Mt5KlineInterval};
    use crate::ExchangeClient;
    use event_center::{EventPublisher, Channel, Event};
    use tokio::sync::{broadcast, Mutex};
    use std::sync::Arc;
    use std::collections::HashMap;
    use tracing_subscriber;
    use types::market::{Exchange, KlineInterval, Symbol};
    use types::order::{CreateOrderParams, FuturesOrderSide, OrderType, GetTransactionDetailParams};
    use types::position::{GetPositionParam, GetPositionNumberParams, PositionSide};
    use types::strategy::TimeRange;
    use std::any::Any;
    use tracing::Level;
    use crate::{ExchangeClientError, metatrader5::mt5_error::Mt5Error};


    const TEST_SYMBOL: &str = "BTCUSDm";
    const TEST_PORT: u16 = 8001;
    const LOGIN: i64 = 76898751;
    const PASSWORD: &str = "HhazJ520....";
    const SERVER: &str = "Exness-MT5Trial5";
    const TERMINAL_PATH: &str = "D:\\Program Files\\MetaTrader\\MetaTrader 5-1\\terminal64.exe";
    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .with_test_writer()
            .try_init();
    }

    fn create_test_event_publisher() -> EventPublisher {
        let channels = Arc::new(Mutex::new(HashMap::<Channel, broadcast::Sender<Event>>::new()));
        EventPublisher::new(channels)
    }

    fn create_test_metatrader5() -> MetaTrader5 {
        let event_publisher = create_test_event_publisher();
        MetaTrader5::new(
            1001,
            LOGIN,
            PASSWORD.to_string(),
            SERVER.to_string(),
            TERMINAL_PATH.to_string(),
            event_publisher,
        )
    }

    #[test]
    fn test_metatrader5_new() {
        init_tracing();
        let mt5 = create_test_metatrader5();

        assert_eq!(mt5.terminal_id, 1001);
        assert_eq!(mt5.login, LOGIN);
        assert_eq!(mt5.password, PASSWORD);
        assert_eq!(mt5.server, SERVER);
        assert_eq!(mt5.terminal_path, TERMINAL_PATH);
    }


    #[tokio::test]
    async fn test_create_mt5_http_client() {
        init_tracing();
        let mut mt5 = create_test_metatrader5();
        
        // Test creating HTTP client
        let result = mt5.create_mt5_http_client(TEST_PORT).await;
        assert!(result.is_ok());
        
        // Verify client was created
        let client_guard = mt5.mt5_http_client.lock().await;
        assert!(client_guard.is_some());
    }

    #[tokio::test]
    async fn test_check_server_start_success_no_http_client() {
        // Initialize tracing subscriber to see logs during test
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .try_init();
        
        let mut mt5 = create_test_metatrader5();
        
        // Without initializing HTTP client, check should fail immediately
        let result = mt5.check_server_start_success().await;
        
        // The method should return false when no HTTP client is initialized
        assert!(!result, "Method should return false when no HTTP client is initialized");
    }

    // Note: Testing start_mt5_server would require mocking the actual MT5 executable
    // and process creation, which is complex and beyond unit testing scope.
    // These would be better suited for integration tests.

    #[test]
    fn test_metatrader5_clone() {
        let mt5 = create_test_metatrader5();
        let mt5_clone = mt5.clone();
        
        assert_eq!(mt5.terminal_id, mt5_clone.terminal_id);
        assert_eq!(mt5.login, mt5_clone.login);
        assert_eq!(mt5.password, mt5_clone.password);
        assert_eq!(mt5.server, mt5_clone.server);
        assert_eq!(mt5.terminal_path, mt5_clone.terminal_path);
        assert_eq!(mt5.process_name, mt5_clone.process_name);
        assert_eq!(mt5.server_port, mt5_clone.server_port);
    }

    #[test]
    fn test_metatrader5_debug() {
        let mt5 = create_test_metatrader5();
        let debug_output = format!("{:?}", mt5);
        
        assert!(debug_output.contains("MetaTrader5"));
        assert!(debug_output.contains(format!("login: {}", LOGIN).as_str()));
    }

    #[test]
    fn test_as_any() {
        let mt5 = create_test_metatrader5();
        let any_ref: &dyn Any = mt5.as_any();
        assert!(any_ref.downcast_ref::<MetaTrader5>().is_some());
    }

    #[test]
    fn test_as_any_mut() {
        let mut mt5 = create_test_metatrader5();
        let any_ref: &mut dyn Any = mt5.as_any_mut();
        assert!(any_ref.downcast_mut::<MetaTrader5>().is_some());
    }

    #[test]
    fn test_clone_box() {
        let mt5 = create_test_metatrader5();
        let cloned = mt5.clone_box();
        
        // Verify the cloned box contains a MetaTrader5 instance
        assert!(cloned.as_any().downcast_ref::<MetaTrader5>().is_some());
    }

    #[test]
    fn test_exchange_type() {
        let mt5 = create_test_metatrader5();
        let exchange_type = mt5.exchange_type();
        
        match exchange_type {
            Exchange::Metatrader5(server) => {
                assert_eq!(server, SERVER);
            }
            _ => panic!("Expected Metatrader5 exchange type"),
        }
    }

    #[tokio::test]
    async fn test_get_ticker_price() {
        let mt5 = create_test_metatrader5();
        let result = mt5.get_ticker_price(TEST_SYMBOL).await;
        
        // Currently returns null, so we test that it doesn't error
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_get_kline_series_without_http_client() {
        let mt5 = create_test_metatrader5();
        let result = mt5.get_kline_series(TEST_SYMBOL, KlineInterval::Minutes1, 100).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        assert!(matches!(result, Err(ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_)))));
    }

    #[tokio::test]
    async fn test_get_kline_series_with_http_client() {
        init_tracing();

        let mut mt5 = create_test_metatrader5();
        let result = mt5.create_mt5_http_client(TEST_PORT).await;
        assert!(result.is_ok());
        let result = mt5.get_kline_series(TEST_SYMBOL, KlineInterval::Minutes1, 2).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_websocket_no_server() {
        let mut mt5 = create_test_metatrader5();
        
        // This will fail because no actual MT5 server is running
        // But we test that the method exists and returns appropriate error
        let result = mt5.connect_websocket().await;
        // The method currently panics on failure, so we expect it to fail
        // In a real implementation, this should return Result<(), String>
    }

    #[tokio::test]
    async fn test_subscribe_kline_stream_no_websocket() {
        let mt5 = create_test_metatrader5();
        let result = mt5.subscribe_kline_stream(TEST_SYMBOL, KlineInterval::Minutes1, 1000).await;
        
        // Should succeed but do nothing since no websocket is connected
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unsubscribe_kline_stream_no_websocket() {
        let mt5 = create_test_metatrader5();
        let result = mt5.unsubscribe_kline_stream(TEST_SYMBOL, KlineInterval::Minutes1, 1000).await;
        
        // Should succeed but do nothing since no websocket is connected
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_socket_stream() {
        let mt5 = create_test_metatrader5();
        let result = mt5.get_socket_stream().await;
        
        // Should succeed even without websocket (returns early if already processing)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_kline_history_no_client() {
        init_tracing();
        let mt5 = create_test_metatrader5();
        let time_range = TimeRange {
            start_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 2).unwrap(),
        };
        
        let result = mt5.get_kline_history(TEST_SYMBOL, KlineInterval::Minutes1, time_range).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        let error = result.unwrap_err();
        tracing::info!("error: {:?}", error);
        assert!(matches!(error, ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_))));
    }

    #[tokio::test]
    async fn test_get_kline_history_with_client() {
        init_tracing();
        let mut mt5 = create_test_metatrader5();
        let result = mt5.create_mt5_http_client(TEST_PORT).await;
        assert!(result.is_ok());
        let time_range = TimeRange {
            start_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 2).unwrap(),
        };
        
        let result = mt5.get_kline_history(TEST_SYMBOL, KlineInterval::Minutes1, time_range).await;
        
        // Should succeed
        assert!(result.is_ok());
    }


    #[tokio::test]
    async fn test_get_kline_history_at_server_is_not_ready() {
        init_tracing();
        let mut mt5 = create_test_metatrader5();
        let result = mt5.create_mt5_http_client(TEST_PORT).await;
        assert!(result.is_ok());
        let time_range = TimeRange {
            start_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2022, 1, 2).unwrap(),
        };
        
        let result = mt5.get_kline_history(TEST_SYMBOL, KlineInterval::Minutes1, time_range).await;
        
        // Should succeed
        assert!(result.is_err());
        let error = result.unwrap_err();
        tracing::info!("error: {:?}", error);
        assert!(matches!(error, ExchangeClientError::MetaTrader5(Mt5Error::HttpClient(_))));
    }


    #[tokio::test]
    async fn test_get_symbols() {
        init_tracing();
        let mut mt5 = create_test_metatrader5();
        let _ = mt5.create_mt5_http_client(TEST_PORT).await;
        let result = mt5.get_symbol_list().await;
        let symbols = result.unwrap();
        assert!(!symbols.is_empty());
        assert!(symbols.contains(&Symbol::new(TEST_SYMBOL, None, None, Exchange::Metatrader5(SERVER.to_string()))));
        

    }

    #[tokio::test]
    async fn test_create_order_no_client() {
        let mt5 = create_test_metatrader5();
        let params = CreateOrderParams {
            strategy_id: 1,
            node_id: "test_node".to_string(),
            account_id: 1,
            exchange: Exchange::Metatrader5("test_server".to_string()),
            symbol: "EURUSD".to_string(),
            order_type: OrderType::Market,
            order_side: FuturesOrderSide::OpenLong,
            quantity: 0.01,
            price: 1.0,
            tp: None,
            sl: None,
            comment: "test".to_string(),
        };
        
        let result = mt5.create_order(params).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_))));
    }

    #[tokio::test]
    async fn test_get_transaction_detail_no_client() {
        let mt5 = create_test_metatrader5();
        let params = GetTransactionDetailParams {
            strategy_id: 1,
            node_id: "test_node".to_string(),
            exchange: Exchange::Metatrader5("test_server".to_string()),
            symbol: TEST_SYMBOL.to_string(),
            transaction_id: Some(123456),
            position_id: None,
            order_id: None,
        };
        
        let result = mt5.get_transaction_detail(params).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        assert!(matches!(result, Err(ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_)))));
    }

    #[tokio::test]
    async fn test_get_transaction_detail_no_ids() {
        let mt5 = create_test_metatrader5();
        let params = GetTransactionDetailParams {
            strategy_id: 1,
            node_id: "test_node".to_string(),
            exchange: Exchange::Metatrader5("test_server".to_string()),
            symbol: TEST_SYMBOL.to_string(),
            transaction_id: None,
            position_id: None,
            order_id: None,
        };
        
        let result = mt5.get_transaction_detail(params).await;
        
        // Should fail because no ID is provided
        assert!(result.is_err());
        // assert!(matches!(result, Err(ExchangeClientError::MetaTrader5(Mt5Error::InvalidParameters(_)))));
    }

    #[tokio::test]
    async fn test_get_position_no_client() {
        let mt5 = create_test_metatrader5();
        let params = GetPositionParam {
            strategy_id: 1,
            node_id: "test_node".to_string(),
            exchange: Exchange::Metatrader5("test_server".to_string()),
            position_id: 123456,
        };
        
        let result = mt5.get_position(params).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        assert!(matches!(result, Err(ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_)))));
    }

    

    #[tokio::test]
    async fn test_get_position_number_no_client() {
        let mt5 = create_test_metatrader5();
        let params = GetPositionNumberParams {
            exchange: Exchange::Metatrader5("test_server".to_string()),
            symbol: TEST_SYMBOL.to_string(),
            position_side: Some(PositionSide::Long),
        };
        
        let result = mt5.get_position_number(params).await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        assert!(matches!(result, Err(ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_)))));
    }

    #[tokio::test]
    async fn test_get_account_info_no_client() {
        let mt5 = create_test_metatrader5();
        let result = mt5.get_account_info().await;
        
        // Should fail because HTTP client is not initialized
        assert!(result.is_err());
        assert!(matches!(result, Err(ExchangeClientError::MetaTrader5(Mt5Error::Initialization(_)))));
    }

    // Test with mock HTTP client (this would require dependency injection or mocking framework)
    // For now, we focus on testing the error cases and basic functionality

    #[test]
    fn test_kline_interval_conversion() {
        // Test that KlineInterval converts properly to Mt5KlineInterval
        let intervals = vec![
            KlineInterval::Minutes1,
            KlineInterval::Minutes5,
            KlineInterval::Minutes15,
            KlineInterval::Minutes30,
            KlineInterval::Hours1,
            KlineInterval::Hours4,
            KlineInterval::Days1,
        ];
        
        for interval in intervals {
            let mt5_interval = Mt5KlineInterval::from(interval);
            // Verify conversion doesn't panic
            let _interval_string = mt5_interval.to_string();
        }
    }
}