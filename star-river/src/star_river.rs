use data_cache::CacheEngine;
use event_center::{EventCenter, Channel};
use heartbeat::Heartbeat;
use indicator_engine::IndicatorEngine;
use database::DatabaseManager;

use crate::market_engine::MarketDataEngine;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::extract::State;




#[derive(Clone)]
pub struct StarRiver {
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub market_engine: Arc<Mutex<MarketDataEngine>>,
    pub event_center: Arc<Mutex<EventCenter>>,
    pub cache_engine: Arc<Mutex<CacheEngine>>,
    pub indicator_engine: Arc<Mutex<IndicatorEngine>>,
    pub database: Arc<Mutex<DatabaseManager>>,
}

impl StarRiver {
    pub async fn new() -> Self {
        
        let event_center = EventCenter::new();
        // 初始化缓存引擎
        let cache_engine_event_publisher = event_center.get_publisher1();
        let exchange_event_receiver = event_center.subscribe(Channel::Exchange).unwrap();
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let indicator_event_receiver = event_center.subscribe(Channel::Indicator).unwrap();
        let cache_engine = Arc::new(Mutex::new(CacheEngine::new( exchange_event_receiver, indicator_event_receiver, command_event_receiver,cache_engine_event_publisher)));
        // 初始化指标引擎
        let indicator_engine_event_publisher = event_center.get_publisher1();
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let response_event_receiver = event_center.subscribe(Channel::Response).unwrap();
        let indicator_engine = Arc::new(Mutex::new(IndicatorEngine::new(command_event_receiver, response_event_receiver, indicator_engine_event_publisher)));
        
        // 初始化数据库
        let command_event_receiver = event_center.subscribe(Channel::Command).unwrap();
        let database_event_publisher = event_center.get_publisher1();
        let database = Arc::new(Mutex::new(DatabaseManager::new(command_event_receiver, database_event_publisher).await));
        Self { 
            heartbeat: Arc::new(Mutex::new(Heartbeat::new(1000))),
            market_engine: Arc::new(Mutex::new(MarketDataEngine::new())),
            event_center: Arc::new(Mutex::new(event_center)),
            cache_engine: cache_engine,
            indicator_engine: indicator_engine,
            database: database,
        }
    }
}


pub async fn init_app(State(app_state): State<StarRiver>) {

    start_heartbeat(State(app_state.clone())).await;
    start_database(State(app_state.clone())).await;
    start_cache_engine(State(app_state.clone())).await;
    start_indicator_engine(State(app_state.clone())).await;
}


async fn start_heartbeat(star_river: State<StarRiver>) {
    let heartbeat = star_river.heartbeat.clone();
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
        tracing::info!("心跳已启动");
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


