#![allow(unused_variables, unused_imports, dead_code)]
use exchange_client::binance::{BinanceExchange, BinanceKlineInterval};
use exchange_client::binance::binance_http_client::BinanceHttpClient;
use exchange_client::binance::binance_ws_client::BinanceWsClient;
use futures::StreamExt;
use exchange_client::binance::market_stream::klines;
use exchange_client::ExchangeClient;
use event_center::EventCenter;
use event_center::Channel;
use indicator_engine::IndicatorEngine;
use strategy::condition_node::ConditionNode;
use tokio::sync::Mutex;
use std::sync::Arc;
use types::indicator::Indicators;
use types::indicator_config::SMAConfig;
use types::market::{Exchange, KlineInterval};
use data_cache::CacheEngine;
use tracing::{event, Level};
use tracing_subscriber;
use tokio::sync::mpsc;
use strategy::strategy::Strategy;
use strategy::condition_node::ConditionType;
use strategy::condition_node::Condition;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    // if std::env::var_os("RUST_LOG").is_none() {
    //     std::env::set_var("RUST_LOG", "info");


    // tracing_subscriber::fmt::init();
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        // build but do not install the subscriber.
        .init();

    

    let event_center = Arc::new(Mutex::new(EventCenter::new()));


    let cache_engine_publisher = {
        let event_center = event_center.lock().await;
        event_center.get_publisher1()
    };

    let exchange_event_receiver = event_center.lock().await.subscribe(Channel::Exchange).unwrap();
    let indicator_event_receiver = event_center.lock().await.subscribe(Channel::Indicator).unwrap();
    let command_event_receiver = event_center.lock().await.subscribe(Channel::Command).unwrap();


    let cache_engine = Arc::new(Mutex::new(CacheEngine::new(exchange_event_receiver, indicator_event_receiver, command_event_receiver, cache_engine_publisher)));


    let indicator_engine_publisher = {
        let event_center = event_center.lock().await;
        event_center.get_publisher1()
    };
    let indicator_engine = Arc::new(Mutex::new(IndicatorEngine::new(indicator_engine_publisher)));


    let binance_publisher = {
        let event_center = event_center.lock().await;
        event_center.get_publisher1()
    };
    let binance_exchange = Arc::new(Mutex::new(BinanceExchange::new(binance_publisher)));
    

    tokio::spawn(async move {
        let mut binance_exchange = binance_exchange.lock().await;
        binance_exchange.init_exchange().await.unwrap();
        binance_exchange.get_kline_series("BTCUSDT", KlineInterval::Minutes1, Some(20), None, None).await.unwrap();

        binance_exchange.subscribe_kline_stream("BTCUSDT", KlineInterval::Minutes1).await.unwrap();

        binance_exchange.get_socket_stream().await.unwrap();

    });



    let cache_engine_clone = cache_engine.clone();
    let market_event_receiver = {
        let event_center = event_center.lock().await;
        event_center.subscribe(Channel::Market).unwrap()
    };
    let command_event_receiver = {
        let event_center = event_center.lock().await;
        event_center.subscribe(Channel::Command).unwrap()
    };

    let event_center_clone = event_center.clone();
    tokio::spawn(async move {
        
        let mut cache_engine = cache_engine_clone.lock().await;
        cache_engine.start().await;
    });

    // let node_event_publisher = {
    //     let event_center = event_center.lock().await;
    //     event_center.get_publisher1()
    // };
    // let response_event_receiver = {
    //     let event_center = event_center.lock().await;
    //     event_center.subscribe(Channel::Response).unwrap()
    // };
    // tokio::spawn(async move {
    //     let mut strategy = Strategy::new("test_strategy".to_string());
    //     let data_source_node_id = strategy.add_data_source_node("BTCUSDT".to_string(), Exchange::Binance, "BTCUSDT".to_string(), KlineInterval::Minutes1, market_event_receiver);
    //     let sma_config = SMAConfig { period: 3 };
    //     let indicator_node_id = strategy.add_indicator_node("SMA14".to_string(), Exchange::Binance, "BTCUSDT".to_string(), KlineInterval::Minutes1, Indicators::SimpleMovingAverage(sma_config), node_event_publisher, response_event_receiver);
        
    //     let mut condition_node = ConditionNode::new("condition_node".to_string(), ConditionType::And);
    //     condition_node.add_condition(Condition::new(data_source_node_id, ">", indicator_node_id));
    //     let condition_node_id = strategy.add_condition_node(condition_node);

    //     strategy.add_edge(&data_source_node_id, &indicator_node_id).await;
    //     strategy.add_edge(&indicator_node_id, &condition_node_id).await;
    //     strategy.run().await;
    // });




    // let indicator_engine = indicator_engine.clone();
    // let market_event_receiver = {
    //     let event_center = event_center.lock().await;
    //     event_center.subscribe(Channel::Market).unwrap()
    // };
    // let command_event_receiver = {
    //     let event_center = event_center.lock().await;
    //     event_center.subscribe(Channel::Command).unwrap()
    // };
    // let response_event_receiver = {
    //     let event_center = event_center.lock().await;
    //     event_center.subscribe(Channel::Response).unwrap()
    // };
    // tokio::spawn(async move {
    //     let indicator_engine = indicator_engine.lock().await;
    //     indicator_engine.start(command_event_receiver, response_event_receiver).await;
    // });

    // 保持主程序运行
    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    Ok(())

}
