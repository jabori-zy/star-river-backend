use data_cache::CacheEngine;
use event_center::{EventCenter, Channel};
use heartbeat::Heartbeat;
use indicator_engine::IndicatorEngine;

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
}

impl StarRiver {
    pub fn new() -> Self {
        
        let event_center = EventCenter::new();
        let event_publisher = event_center.get_publisher1();
        let cache_engine = Arc::new(Mutex::new(CacheEngine::new(event_publisher.clone())));
        
        Self { 
            heartbeat: Arc::new(Mutex::new(Heartbeat::new(1000))),
            market_engine: Arc::new(Mutex::new(MarketDataEngine::new())),
            event_center: Arc::new(Mutex::new(event_center)),
            cache_engine: cache_engine.clone(),
            indicator_engine: Arc::new(Mutex::new(IndicatorEngine::new(event_publisher))),
        }
    }
}


pub async fn init_app(State(app_state): State<StarRiver>) {
    start_heartbeat(State(app_state.clone())).await;
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
    let exchange_event_receiver = star_river.event_center.lock().await.subscribe(Channel::Exchange).expect("订阅交易所通道失败");
    let indicator_event_receiver = star_river.event_center.lock().await.subscribe(Channel::Indicator).expect("订阅指标通道失败");
    let command_event_receiver = star_river.event_center.lock().await.subscribe(Channel::Command).expect("订阅命令通道失败");
    let cache_engine = star_river.cache_engine.clone();
    tokio::spawn(async move {
        let mut cache_engine = cache_engine.lock().await;
        cache_engine.start(exchange_event_receiver, command_event_receiver, indicator_event_receiver).await;
    });
}

async fn start_indicator_engine(star_river: State<StarRiver>) {
    let market_event_receiver = star_river.event_center.lock().await.subscribe(Channel::Market).unwrap();
    let response_event_receiver = star_river.event_center.lock().await.subscribe(Channel::Response).unwrap();
    let indicator_engine = star_river.indicator_engine.clone();
    tokio::spawn(async move {
        let indicator_engine = indicator_engine.lock().await;
        indicator_engine.start(market_event_receiver, response_event_receiver).await;
    });
}


