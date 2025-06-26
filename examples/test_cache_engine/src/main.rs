
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
use event_center::command::market_engine_command::MarketEngineCommand;
use types::market::KlineInterval;
use event_center::command::exchange_engine_command::UnregisterExchangeParams;
use types::engine::EngineName;
use engine::exchange_engine::ExchangeEngine;
use exchange_client::metatrader5::MetaTrader5;
use tokio::sync::oneshot;

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


    let event_publisher = event_center.get_event_publisher();
    let command_publisher = event_center.get_command_publisher();
    let engine_manager_clone = engine_manager.clone();
    // 注册交易所
    tokio::spawn(async move {
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        // 注册第一个终端
        let (responder, receiver) = oneshot::channel();
        let register_param = RegisterExchangeParams {
            account_id: 1,
            exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
            sender: "test".to_string(),
            timestamp: 1111,
            responder: responder,
        };
        let command_event = Command::ExchangeEngine(ExchangeEngineCommand::RegisterExchange(register_param));
        command_publisher.send(command_event).await.unwrap();

        // tokio::time::sleep(Duration::from_secs(10)).await;

        // // 获取第一个终端的数据
        let (responder, receiver) = oneshot::channel();
        let command_event = Command::MarketEngine(MarketEngineCommand::SubscribeKlineStream(SubscribeKlineStreamParams {
            strategy_id: 1,
            node_id: "test".to_string(),
            sender: "test".to_string(),
            timestamp: 1111,
            responder: responder,
            account_id: 1,
            exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
            frequency: 2000,
            cache_size: 2,
            interval: KlineInterval::Minutes1,
            symbol: "BTCUSDm".to_string(),
        }));
        command_publisher.send(command_event).await.unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;

        // 停止服务
        // tracing::info!("停止服务");
        // let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::UnregisterExchange(UnregisterExchangeParams {
        //     account_id: 2,
        //     sender: "test".to_string(),
        //     timestamp: 1111,
        //     request_id: Uuid::new_v4(),
        // }));
        // event_publisher.publish(command_event.into()).unwrap();

    //     // 注册第二个终端
    //     let register_param = RegisterMt5ExchangeParams {
    //         terminal_id: 2,
    //         account_id: 76898751,
    //         password: "HhazJ520....".to_string(),
    //         server: "Exness-MT5Trial5".to_string(),
    //         terminal_path: r"D:/Program Files/MetaTrader 5-2/terminal64.exe".to_string(),
    //         sender: "test".to_string(),
    //         timestamp: 1111,
    //         request_id: Uuid::new_v4(),
    //     };
    //     let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::RegisterMt5Exchange(register_param));
    //     event_publisher.publish(command_event.into()).unwrap();




        // tracing::info!("创建订单");
        // let params = CreateOrderParams {
        //     base_params: BaseCommandParams {
        //         strategy_id: 1,
        //         node_id: "test".to_string(),
        //         sender: "test".to_string(),
        //         timestamp: 1111,
        //         request_id: Uuid::new_v4(),
        //     },
        //     account_id: 1,
        //     exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
        //     symbol: "BTCUSDm".to_string(),
        //     order_type: OrderType::Market,
        //     order_side: OrderSide::Long,
        //     quantity: 0.01,
        //     price: 81550.00,
        //     tp: None,
        //     sl: None,
        //     comment: "111".to_string(),
        // };
        // let exchange_engine = engine_manager_clone.lock().await.get_engine(EngineName::ExchangeEngine).await;
        // let mut exchange_engine = exchange_engine.lock().await;
        // let exchange_engine = exchange_engine.as_any_mut().downcast_mut::<ExchangeEngine>().unwrap();
        
        
        // let mut mt5_exchange = exchange_engine.get_exchange(&1).await.unwrap();
        // let mt5_exchange = mt5_exchange.as_any_mut().downcast_mut::<MetaTrader5>().unwrap();
        // mt5_exchange.create_order(params).await.unwrap();
        

    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;



}



