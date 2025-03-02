#![allow(unused_variables, unused_imports)]
use exchange_client::binance::{BinanceExchange, BinanceKlineInterval};
use exchange_client::binance::binance_http_client::BinanceHttpClient;
use exchange_client::binance::binance_ws_client::BinanceWsClient;
use futures::StreamExt;
use exchange_client::binance::market_stream::klines;
use exchange_client::ExchangeClient;
use event_center::EventCenter;
use event_center::Channel;
use indicator_engine::IndicatorEngine;
use tokio::sync::Mutex;
use std::sync::Arc;
use types::indicator::Indicators;
use types::indicator_config::SMAConfig;
use types::market::{Exchange, KlineInterval};
use data_cache::CacheEngine;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    // 设置tracing日志为debug
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish());
    

    let event_center = Arc::new(Mutex::new(EventCenter::new()));
    let binance_publisher = {
        let event_center = event_center.lock().await;
        event_center.get_publisher(Channel::Market).unwrap()
    };


    // let mut indicator_engine = IndicatorEngine::new(event_center_clone1.clone());

    let binance_exchange = Arc::new(Mutex::new(BinanceExchange::new(binance_publisher)));
    binance_exchange.lock().await.init_exchange().await.unwrap();
    // let cache_engine = Arc::new(Mutex::new(CacheEngine::new(event_center.clone())));

    // tokio::spawn(async move {
    //     let mut binance_exchange = binance_exchange.lock().await;
    //     binance_exchange.init_exchange().await.unwrap();
        // binance_exchange.get_kline_series("BTCUSDT", KlineInterval::Minutes1, Some(2), None, None).await.unwrap();
        // binance_exchange.get_kline_series("BTCUSDT", KlineInterval::Minutes1, Some(2), None, None).await.unwrap();

        // binance_exchange.subscribe_kline_stream("BTCUSDT", KlineInterval::Minutes1).await.unwrap();

        // binance_exchange.get_socket_stream().await.unwrap();
    // });

    // let cache_manager_clone = cache_manager.clone();
    // tokio::spawn(async move {
    //     let mut cache_manager = cache_manager_clone.lock().await;
    //     cache_manager.command_listener().await;
    // });

    // let cache_engine_clone = cache_engine.clone();
    // tokio::spawn(async move {
    //     let mut cache_engine = cache_engine_clone.lock().await;
    //     cache_engine.start().await;
    // });




    // 保持主程序运行
    // tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    Ok(())

}
