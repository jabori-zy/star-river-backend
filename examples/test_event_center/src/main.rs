
use event_center::command::market_engine_command::SubscribeKlineStreamParams;
use event_center::{Channel, EventCenter};
use database::DatabaseManager;
use exchange_client::ExchangeClient;
use sea_orm::prelude::Uuid;
use types::market::Exchange;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use event_center::command::Command;
use engine::engine_manager::EngineManager;
use event_center::command::exchange_engine_command::RegisterExchangeParams;
use event_center::command::exchange_engine_command::ExchangeEngineCommand;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::order::{OrderType, FuturesOrderSide};
use event_center::command::exchange_engine_command::RegisterMt5ExchangeParams;
use event_center::command::market_engine_command::MarketEngineCommand;
use types::market::KlineInterval;
use event_center::command::exchange_engine_command::UnregisterExchangeParams;
use types::engine::EngineName;
use engine::exchange_engine::ExchangeEngine;
use exchange_client::metatrader5::MetaTrader5;
use tokio::sync::oneshot;
use event_center::command::cache_engine_command::CacheEngineCommand;
use event_center::command::cache_engine_command::AddCacheKeyParams;
use types::cache::Key;
use types::cache::key::KlineKey;

#[tokio::main]
async fn main() {

    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // build but do not install the subscriber.
        .init();

    let mut event_center = EventCenter::new().init_channel().await;

    let database = DatabaseManager::new().await;

    // 初始化数据库
    let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100)));

    let engine_manager = Arc::new(Mutex::new(EngineManager::new(
        &mut event_center,
        database.get_conn(),
        heartbeat.clone()
    ).await));

    // 启动心跳
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
    });

    // 启动市场引擎
    let engine_manager_clone = engine_manager.clone();

    
    tokio::spawn(async move {
        let engine_manager = engine_manager_clone.lock().await;
        engine_manager.start().await;
    });

    tokio::spawn(async move {
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        let (resp_tx, resp_rx) = oneshot::channel();
        let add_cache_key_command = CacheEngineCommand::AddCacheKey(AddCacheKeyParams {
            strategy_id: 1,
            key: Key::Kline(KlineKey::new(Exchange::Metatrader5("Exness-MT5Trial5".to_string()), "BTCUSDT".to_string(), KlineInterval::Minutes1, None, None)),
            max_size: Some(100),
            duration: Duration::from_secs(1000),
            sender: "test".to_string(),
            timestamp: 1111,
            responder: resp_tx,
        });
        let command_event = Command::CacheEngine(add_cache_key_command);
        event_center.get_command_publisher().send(command_event.into()).await.unwrap();

        let response_event = resp_rx.await.unwrap();
        tracing::debug!("市场数据引擎添加缓存key成功, response: {:?}", response_event);
    });




    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;


}



