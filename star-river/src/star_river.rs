use data_cache::CacheEngine;
use event_center::{EventCenter, Channel};
use heartbeat::Heartbeat;
use indicator_engine::IndicatorEngine;
use database::DatabaseManager;
use exchange_client::ExchangeManager;
use market_engine::MarketDataEngine;
use strategy_engine::engine::StrategyEngine;

use tokio::sync::Mutex;
use std::sync::Arc;
use axum::extract::State;




#[derive(Clone)]
pub struct StarRiver {
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub exchange_manager: Arc<Mutex<ExchangeManager>>,
    pub market_engine: Arc<Mutex<MarketDataEngine>>,
    pub event_center: Arc<Mutex<EventCenter>>,
    pub cache_engine: Arc<Mutex<CacheEngine>>,
    pub indicator_engine: Arc<Mutex<IndicatorEngine>>,
    pub database: Arc<Mutex<DatabaseManager>>,
    pub strategy_engine: Arc<Mutex<StrategyEngine>>,
}

impl StarRiver {
    pub async fn new() -> Self {
        
        let event_center = EventCenter::new();
        // 初始化exchange manager
        let exchange_manager_event_publisher = event_center.get_publisher();
        let exchange_manager_command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let exchange_manager_response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
        let exchange_manager = Arc::new(Mutex::new(ExchangeManager::new(exchange_manager_event_publisher, exchange_manager_command_event_receiver, exchange_manager_response_event_receiver)));
        // 初始化市场引擎
        let market_engine_event_publisher = event_center.get_publisher();
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
        let market_engine = MarketDataEngine::new(market_engine_event_publisher, command_event_receiver, response_event_receiver, exchange_manager.clone());
        // 初始化缓存引擎
        let cache_engine_event_publisher = event_center.get_publisher();
        let exchange_event_receiver = event_center.subscribe(Channel::Exchange).unwrap();
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let indicator_event_receiver = event_center.subscribe(Channel::Indicator).unwrap();
        let cache_engine = CacheEngine::new( exchange_event_receiver, indicator_event_receiver, command_event_receiver,cache_engine_event_publisher);
        // 初始化指标引擎
        let indicator_engine_event_publisher = event_center.get_publisher();
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
        let indicator_engine = IndicatorEngine::new(command_event_receiver, response_event_receiver, indicator_engine_event_publisher);
        
        // 初始化数据库
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let database_event_publisher = event_center.get_publisher();
        let database = DatabaseManager::new(command_event_receiver, database_event_publisher).await;

        // 初始化策略引擎
        let market_event_receiver = event_center.subscribe(Channel::Market).unwrap();
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
        let strategy_event_receiver = event_center.subscribe(Channel::Strategy).unwrap();
        let strategy_engine_event_publisher = event_center.get_publisher();
        let database_conn = database.get_conn();
        let strategy_engine = StrategyEngine::new(
            market_event_receiver, 
            command_event_receiver, 
            response_event_receiver,
            strategy_event_receiver,
            strategy_engine_event_publisher, 
            database_conn
        );

        Self { 
            heartbeat: Arc::new(Mutex::new(Heartbeat::new(1000))),
            exchange_manager: exchange_manager,
            market_engine: Arc::new(Mutex::new(market_engine)),
            event_center: Arc::new(Mutex::new(event_center)),
            cache_engine: Arc::new(Mutex::new(cache_engine)),
            indicator_engine: Arc::new(Mutex::new(indicator_engine)),
            database: Arc::new(Mutex::new(database)),
            strategy_engine: Arc::new(Mutex::new(strategy_engine)),
        }
    }
}


pub async fn init_app(State(app_state): State<StarRiver>) {

    start_heartbeat(State(app_state.clone())).await;
    start_database(State(app_state.clone())).await;
    start_exchange_manager(State(app_state.clone())).await;
    start_market_engine(State(app_state.clone())).await;
    start_cache_engine(State(app_state.clone())).await;
    start_indicator_engine(State(app_state.clone())).await;
    start_strategy_engine(State(app_state.clone())).await;
}


async fn start_heartbeat(star_river: State<StarRiver>) {
    let heartbeat = star_river.heartbeat.clone();
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
        tracing::info!("心跳已启动");
    });
}

async fn start_exchange_manager(star_river: State<StarRiver>) {
    let exchange_manager = star_river.exchange_manager.clone();
    tokio::spawn(async move {
        let exchange_manager = exchange_manager.lock().await;
        exchange_manager.start().await.unwrap();
    });
}

async fn start_cache_engine(star_river: State<StarRiver>) {
    let cache_engine = star_river.cache_engine.clone();
    tokio::spawn(async move {
        let mut cache_engine = cache_engine.lock().await;
        cache_engine.start().await;
    });
}

async fn start_indicator_engine(star_river: State<StarRiver>) {
    let indicator_engine = star_river.indicator_engine.clone();
    tokio::spawn(async move {
        let indicator_engine = indicator_engine.lock().await;
        indicator_engine.start().await;
    });
}

async fn start_database(star_river: State<StarRiver>) {
    let database = star_river.database.clone();
    tokio::spawn(async move {
        let database = database.lock().await;
        database.start().await;
    });
}

async fn start_market_engine(star_river: State<StarRiver>) {
    let market_engine = star_river.market_engine.clone();
    tokio::spawn(async move {
        let mut market_engine = market_engine.lock().await;
        market_engine.start().await.unwrap();
    });
}

async fn start_strategy_engine(star_river: State<StarRiver>) {
    let strategy_engine = star_river.strategy_engine.clone();
    tokio::spawn(async move {
        let strategy_engine = strategy_engine.lock().await;
        strategy_engine.start().await.unwrap();
    });
}


