use event_center::EventCenter;
use strategy_engine::engine::StrategyEngine;
use database::DatabaseManager;
use event_center::Channel;
use tracing::Level;
use tracing_subscriber;
use strategy_engine::strategy::Strategy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::fmt()
    // filter spans/events with level TRACE or higher.
    .with_max_level(Level::INFO)
    // build but do not install the subscriber.
    .init();

    // 初始化事件中心
    let event_center = EventCenter::new();

    // 初始化数据库
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let event_publisher = event_center.get_publisher1();
    let database_manager = DatabaseManager::new(command_event_receiver, event_publisher).await;

    // 初始化策略引擎
    let strategy_engine_event_publisher = event_center.get_publisher1();
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
    let market_event_receiver = event_center.subscribe(Channel::Market).unwrap();
    let database = database_manager.get_conn();
    let mut strategy_engine = StrategyEngine::new(market_event_receiver, command_event_receiver, response_event_receiver, strategy_engine_event_publisher, database);

    tokio::spawn(async move {
        database_manager.start().await;
        
    });

    let market_event_receiver = event_center.subscribe(Channel::Market).unwrap();
    tokio::spawn(async move {
        strategy_engine.start().await.unwrap();
        println!("策略引擎启动成功");
        // strategy_engine.create_strategy("test".to_string(), "test_description".to_string()).await.unwrap();
        let strategy_info = strategy_engine.get_strategy_by_id(9).await.unwrap();
        let strategy_id = strategy_engine.create_strategy_by_info(strategy_info).await.unwrap();
        strategy_engine.run_strategy(strategy_id).await.unwrap();
        
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

    Ok(())
    
}


