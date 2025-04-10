
use event_center::{Channel, EventCenter};
use database::DatabaseManager;
use sea_orm::prelude::Uuid;
use types::{market::Exchange, order::OrderRequest};
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use event_center::request_event::RegisterExchangeParams;
use event_center::request_event::CommandEvent;
use engine::engine_manager::EngineManager;
use event_center::request_event::ExchangeManagerCommand;

#[tokio::main]
async fn main() {

    let filter = EnvFilter::new("debug,reqwest=warn");
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // build but do not install the subscriber.
        .init();

    let event_center = EventCenter::new();


    let engine_event_publisher = event_center.get_event_publisher();
    let market_event_receiver = event_center.subscribe(&Channel::Market).unwrap();
    let request_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();


    // 初始化数据库
    let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).unwrap();
    let command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let event_publisher = event_center.get_event_publisher();
    let database = DatabaseManager::new(command_event_receiver, event_publisher);

    let engine_manager = EngineManager::new(
        engine_event_publisher, 
        exchange_event_receiver,
        market_event_receiver, 
        request_event_receiver, 
        response_event_receiver,
        database.await.get_conn()
    );

    // 启动市场引擎
    tokio::spawn(async move {
        engine_manager.start().await;
    });


    let event_publish = event_center.get_event_publisher();
    // 注册交易所
    tokio::spawn(async move {
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        // 注册交易所
        let register_param = RegisterExchangeParams {
            exchange: Exchange::Binance,
            sender: "test".to_string(),
            timestamp: 1111,
            request_id: Uuid::new_v4(),
        };
        let command_event = CommandEvent::ExchangeManager(ExchangeManagerCommand::RegisterExchange(register_param));
        event_publish.publish(command_event.into()).unwrap();

    });


    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;



}



