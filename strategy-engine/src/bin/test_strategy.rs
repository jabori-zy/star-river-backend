use event_center::EventCenter;
use strategy_engine::engine::StrategyEngine;
use database::DatabaseManager;
use event_center::Channel;
use tracing::Level;
use tracing_subscriber;
use market_engine::MarketDataEngine;
use data_cache::CacheEngine;
use indicator_engine::IndicatorEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    tracing_subscriber::fmt()
    // filter spans/events with level TRACE or higher.
    .with_max_level(Level::DEBUG)
    // build but do not install the subscriber.
    .init();

    // 初始化事件中心
    let event_center = EventCenter::new();

    // 初始化数据库
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let event_publisher = event_center.get_publisher();
    let database_manager = DatabaseManager::new(command_event_receiver, event_publisher).await;

    // 初始化市场引擎
    let market_engine_event_publisher = event_center.get_publisher();
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let market_event_receiver = event_center.subscribe(Channel::Market).unwrap();
    let mut market_engine = MarketDataEngine::new(market_engine_event_publisher, command_event_receiver, market_event_receiver);

    // 初始化缓存引擎
    let cache_engine_event_publisher = event_center.get_publisher();
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let exchange_event_receiver = event_center.subscribe(Channel::Exchange).unwrap();
    let indicator_event_receiver = event_center.subscribe(Channel::Indicator).unwrap();
    let mut cache_engine = CacheEngine::new(exchange_event_receiver, indicator_event_receiver, command_event_receiver, cache_engine_event_publisher);

    // 初始化策略引擎
    
    let strategy_engine_event_publisher = event_center.get_publisher();
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
    let market_event_receiver = event_center.subscribe(Channel::Market).unwrap();
    let strategy_event_receiver = event_center.subscribe(Channel::Strategy).unwrap();
    let database = database_manager.get_conn();
    let mut strategy_engine = StrategyEngine::new(market_event_receiver, command_event_receiver, response_event_receiver, strategy_event_receiver, strategy_engine_event_publisher, database);

    // 初始化指标引擎
    let indicator_engine_event_publisher = event_center.get_publisher();
    let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
    let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
    let indicator_engine = IndicatorEngine::new(command_event_receiver, response_event_receiver, indicator_engine_event_publisher);


    tokio::spawn(async move {
        database_manager.start().await;
        
    });

    // 启动市场引擎
    tokio::spawn(async move {
        market_engine.start().await.unwrap();
    });

    // 启动缓存引擎
    tokio::spawn(async move {
        cache_engine.start().await;
    });

    // 启动指标引擎
    tokio::spawn(async move {
        indicator_engine.start().await;
    });

    tokio::spawn(async move {
        strategy_engine.start().await.unwrap();
        // strategy_engine.create_strategy("test".to_string(), "test_description".to_string()).await.unwrap();
        let strategy_info = strategy_engine.get_strategy_by_id(9).await.unwrap();
        let strategy_id = strategy_engine.load_strategy_by_info(strategy_info).await.unwrap();
        strategy_engine.init_strategy(strategy_id).await.unwrap();
        
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

    Ok(())
    
}


