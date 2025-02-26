use data_cache::CacheEngine;
use event_center::{EventCenter, Channel};
use heartbeat::Heartbeat;
use indicator_engine::IndicatorEngine;

use crate::market_engine::MarketDataEngine;
use std::sync::Arc;
use tokio::sync::Mutex;



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
        let datacache_publisher = event_center.get_publisher(Channel::Market).expect("Failed to get datacache publisher");
        let indicator_publisher = event_center.get_publisher(Channel::Indicator).expect("Failed to get indicator publisher");
        let cache_engine = Arc::new(Mutex::new(CacheEngine::new(datacache_publisher)));
        Self { 
            heartbeat: Arc::new(Mutex::new(Heartbeat::new(1000))),
            market_engine: Arc::new(Mutex::new(MarketDataEngine::new())),
            event_center: Arc::new(Mutex::new(event_center)),
            cache_engine: cache_engine.clone(),
            indicator_engine: Arc::new(Mutex::new(IndicatorEngine::new(cache_engine, indicator_publisher))),
        }
    }
}



