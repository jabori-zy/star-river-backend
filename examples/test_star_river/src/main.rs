
use event_center::command_event::market_engine_command::SubscribeKlineStreamParams;
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
use event_center::command_event::exchange_engine_command::RegisterMt5ExchangeParams;
use event_center::command_event::market_engine_command::MarketEngineCommand;
use types::market::KlineInterval;
use event_center::command_event::exchange_engine_command::UnregisterExchangeParams;

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
    let account_event_receiver = event_center.subscribe(&Channel::Account).unwrap();


    // 初始化数据库
    let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100)));

    let engine_manager = EngineManager::new(
        engine_event_publisher, 
        exchange_event_receiver,
        market_event_receiver, 
        request_event_receiver, 
        response_event_receiver,
        order_event_receiver,
        account_event_receiver,
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
        // 注册第一个终端
        let register_param = RegisterExchangeParams {
            account_id: 2,
            exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
            sender: "test".to_string(),
            timestamp: 1111,
            request_id: Uuid::new_v4(),
        };
        let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::RegisterExchange(register_param));
        event_publisher.publish(command_event.into()).unwrap();

        // tokio::time::sleep(Duration::from_secs(10)).await;

        // // 获取第一个终端的数据
        // let command_event = CommandEvent::MarketEngine(MarketEngineCommand::SubscribeKlineStream(SubscribeKlineStreamParams {
        //     strategy_id: 1,
        //     node_id: "test".to_string(),
        //     sender: "test".to_string(),
        //     timestamp: 1111,
        //     request_id: Uuid::new_v4(),
        //     account_id: 1,
        //     exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
        //     frequency: 1000,
        //     interval: KlineInterval::Minutes1,
        //     symbol: "BTCUSDm".to_string(),
        // }));
        // event_publisher.publish(command_event.into()).unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;

        // 停止服务
        tracing::info!("停止服务");
        let command_event = CommandEvent::ExchangeEngine(ExchangeEngineCommand::UnregisterExchange(UnregisterExchangeParams {
            account_id: 2,
            sender: "test".to_string(),
            timestamp: 1111,
            request_id: Uuid::new_v4(),
        }));
        event_publisher.publish(command_event.into()).unwrap();

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
        // let command_event = CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(CreateOrderParams {
        //     base_params: BaseCommandParams {
        //         strategy_id: 1,
        //         node_id: "test".to_string(),
        //         sender: "test".to_string(),
        //         timestamp: 1111,
        //         request_id: Uuid::new_v4(),
        //     },
        //     exchange: Exchange::Metatrader5,
        //     symbol: "BTCUSDm".to_string(),
        //     order_type: OrderType::Market,
        //     order_side: OrderSide::Long,
        //     quantity: 0.01,
        //     price: 81550.00,
        //     tp: None,
        //     sl: None,
        //     comment: "111".to_string(),
        // }));
        // event_publisher.publish(command_event.into()).unwrap();

    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;



}



