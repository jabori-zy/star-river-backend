use exchange_client::metatrader5::MetaTrader5;
use exchange_client::ExchangeClient;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use types::market::KlineInterval;
use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::EventCenter;
use types::order::{OrderType, OrderSide};
use types::market::Exchange;
use types::order::OrderRequest;


#[tokio::main]
async fn main() {
    // 设置生产环境的日志级别
    let filter = EnvFilter::new("debug,reqwest=warn");
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // build but do not install the subscriber.
        .init();

    let event_center = Arc::new(Mutex::new(EventCenter::new()));

    let event_publisher = event_center.lock().await.get_publisher();
    let mt5 = Arc::new(Mutex::new(MetaTrader5::new(event_publisher)));
    let mt5_clone = mt5.clone();
    mt5_clone.lock().await.start_mt5_server(false).await.unwrap();
    let result = mt5_clone.lock().await.ping().await;
    tracing::info!("ping结果: {:?}", result);

    let result = mt5_clone.lock().await.initialize_client().await;
    tracing::info!("初始化结果: {:?}", result);

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let result = mt5_clone.lock().await.get_client_status().await;
    tracing::info!("获取客户端状态结果: {:?}", result);

    mt5_clone.lock().await.login(23643, "HhazJ520!!!!", "EBCFinancialGroupKY-Demo", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
    // mt5_clone.lock().await.login(76898751, "HhazJ520....", "Exness-MT5Trial5", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");


    // mt5_clone.lock().await.connect_websocket().await.unwrap();

    mt5_clone.lock().await.get_kline_series("BTCUSDm", KlineInterval::Minutes1, Some(2)).await.unwrap();
    // mt5_clone.lock().await.subscribe_kline_stream("XAUUSD", KlineInterval::Minutes1, 500).await.unwrap();
    // mt5_clone.lock().await.get_socket_stream().await.unwrap();

    mt5_clone.lock().await.send_order(OrderRequest {
        exchange: Exchange::Metatrader5,
        symbol: "XAUUSD".to_string(),
        order_type: OrderType::Market,
        order_side: OrderSide::Long,
        quantity: 0.01,
        price: 0.00,
        tp: None,
        sl: None,
    }).await.unwrap();
    

    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;

    
}


