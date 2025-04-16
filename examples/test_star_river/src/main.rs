
use event_center::{Channel, EventCenter};
use database::DatabaseManager;
use sea_orm::prelude::Uuid;
use types::market::Exchange;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use event_center::command_event::CommandEvent;
use engine::engine_manager::EngineManager;
use event_center::command_event::exchange_engine_command::RegisterExchangeParams;
use event_center::command_event::exchange_engine_command::ExchangeEngineCommand;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::order::{OrderType, OrderSide};
use event_center::command_event::order_engine_command::CreateOrderParams;
use event_center::command_event::order_engine_command::OrderEngineCommand;
use event_center::command_event::BaseCommandParams;


#[tokio::main]
async fn main() {

    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // build but do not install the subscriber.
        .init();

    let event_center = EventCenter::new();

    let database = DatabaseManager::new().await;


    let engine_event_publisher = event_center.get_event_publisher();
    let market_event_receiver = event_center.subscribe(&Channel::Market).unwrap();
    let request_event_receiver = event_center.subscribe(&Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(&Channel::Response).unwrap();
    let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).unwrap();
    let order_event_receiver = event_center.subscribe(&Channel::Order).unwrap();


    // 初始化数据库
    let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100)));

    let engine_manager = EngineManager::new(
        engine_event_publisher, 
        exchange_event_receiver,
        market_event_receiver, 
        request_event_receiver, 
        response_event_receiver,
        order_event_receiver,
        database.get_conn(),
        heartbeat.clone()
    );

    // 启动心跳
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
    });

    // 启动市场引擎
    tokio::spawn(async move {
        engine_manager.start().await;
    });


    let event_publisher = event_center.get_event_publisher();
    // 注册交易所
    tokio::spawn(async move {
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        // 注册交易所
        let register_param = RegisterExchangeParams {
            exchange: Exchange::Metatrader5,
            sender: "test".to_string(),
            timestamp: 1111,
            request_id: Uuid::new_v4(),
        };
        let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::RegisterExchange(register_param));
        event_publisher.publish(command_event.into()).unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;


        tracing::info!("创建订单");
        let command_event = CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(CreateOrderParams {
            base_params: BaseCommandParams {
                strategy_id: 1,
                node_id: "test".to_string(),
                sender: "test".to_string(),
                timestamp: 1111,
                request_id: Uuid::new_v4(),
            },
            exchange: Exchange::Metatrader5,
            symbol: "BTCUSDm".to_string(),
            order_type: OrderType::Market,
            order_side: OrderSide::Long,
            quantity: 0.01,
            price: 81550.00,
            tp: None,
            sl: None,
            comment: "111".to_string(),
        }));
        event_publisher.publish(command_event.into()).unwrap();

    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;



}



