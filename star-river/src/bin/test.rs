use data_cache::CacheEngine;
use event_center::{request_event::CreateOrderParams, Channel, EventCenter};
use heartbeat::Heartbeat;
use indicator_engine::IndicatorEngine;
use database::DatabaseManager;
use exchange_client::{metatrader5::MetaTrader5, ExchangeManager};
use market_engine::MarketDataEngine;
use sea_orm::prelude::Uuid;
use strategy_engine::engine::StrategyEngine;
use order_engine::OrderEngine;

use tokio::sync::Mutex;
use types::{market::Exchange, order::OrderRequest};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tracing::{event, Level};
use tracing_subscriber::EnvFilter;
use event_center::request_event::{ExchangeManagerCommand, OrderEngineCommand, RegisterExchangeParams};
use event_center::request_event::CommandEvent;

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
    // 初始化exchange manager
    let exchange_manager_event_publisher = event_center.get_event_publisher();
    let exchange_manager_command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let exchange_manager_response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
    let exchange_manager = Arc::new(Mutex::new(ExchangeManager::new(exchange_manager_event_publisher, exchange_manager_command_event_receiver, exchange_manager_response_event_receiver)));

    // 初始化数据库
    let command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let database_event_publisher = event_center.get_event_publisher();
    let database = DatabaseManager::new(command_event_receiver, database_event_publisher).await;

    // 初始化市场引擎
    let market_engine_event_publisher = event_center.get_event_publisher();
    let command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
    let mut market_engine = MarketDataEngine::new(market_engine_event_publisher, command_event_receiver, response_event_receiver, exchange_manager.clone());

    // 初始化订单引擎
    let order_engine_event_publisher = event_center.get_event_publisher();
    let order_engine_command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let order_engine_response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
    let database_conn = database.get_conn();
    let order_engine = OrderEngine::new(order_engine_command_event_receiver, order_engine_response_event_receiver, order_engine_event_publisher, exchange_manager.clone(), database_conn);
    // 初始化缓存引擎
    let cache_engine_event_publisher = event_center.get_event_publisher();
    let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).unwrap();
    let command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let indicator_event_receiver = event_center.subscribe(&Channel::Indicator).unwrap();
    let mut cache_engine = CacheEngine::new( exchange_event_receiver, indicator_event_receiver, command_event_receiver,cache_engine_event_publisher);
    // 初始化指标引擎
    let indicator_engine_event_publisher = event_center.get_event_publisher();
    let command_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
    let indicator_engine = IndicatorEngine::new(command_event_receiver, response_event_receiver, indicator_engine_event_publisher);


    //启动交易所管理器
    tokio::spawn(async move {
        let exchange_manager = exchange_manager.lock().await;
        exchange_manager.start().await.unwrap();
    });

    // 启动市场引擎
    tokio::spawn(async move {
        market_engine.start().await.unwrap();
    });

    // 启动订单引擎
    tokio::spawn(async move {
        order_engine.start().await.unwrap();
    });

    // 启动缓存引擎
    tokio::spawn(async move {
        cache_engine.start().await;
    });

    // 启动指标引擎 
    tokio::spawn(async move {
        indicator_engine.start().await;
    });


    let event_publish = event_center.get_event_publisher();
    // 注册交易所
    tokio::spawn(async move {
        
        // 注册交易所
        let register_param = RegisterExchangeParams {
            exchange: Exchange::Metatrader5,
            sender: "test".to_string(),
            timestamp: 1111,
            request_id: Uuid::new_v4(),
        };
        let command_event = CommandEvent::ExchangeManager(ExchangeManagerCommand::RegisterExchange(register_param));
        event_publish.publish(command_event.into()).unwrap();

        tracing::info!("等待10秒");
        tokio::time::sleep(Duration::from_secs(20)).await;

        // 创建订单命令
        let create_order_params = CreateOrderParams {
            strategy_id: 1,
            node_id: "test".to_string(),
            order_request: OrderRequest {
                strategy_id: 1,
                node_id: "test".to_string(),
                exchange: Exchange::Metatrader5,
                symbol: "BTCUSDm".to_string(),
                order_type: types::order::OrderType::Market,
                order_side: types::order::OrderSide::Long,
                quantity: 0.01,
                price: 0.00,
                tp: None,
                sl: None,
            },
            sender: "test".to_string(),
            timestamp:1111,
            request_id: Uuid::new_v4()
        };
        let command = CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(create_order_params));
        event_publish.publish(command.into()).unwrap();

    });



    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;



}



