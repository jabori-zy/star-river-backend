pub mod cache_engine_context;


use std::fmt::Debug;
use cache_engine_context::CacheEngineContext;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use crate::Engine;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use types::cache::{CacheKey, CacheValue};
use std::time::Duration;
use types::market::{Exchange, KlineInterval,Kline};
use types::cache::cache_key::KlineCacheKey;
use types::cache::cache_key::IndicatorCacheKey;
use types::indicator::Indicator;
use types::indicator::IndicatorConfig;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct CacheEngine {
    pub context: Arc<RwLock<Box<dyn EngineContext>>>,
}


#[async_trait]
impl Engine for CacheEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn EngineContext>>> {
        self.context.clone()
    }
}

impl CacheEngine {
    pub fn new(
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: CommandReceiver,
        exchange_event_receiver: EventReceiver,
    ) -> Self {
        let context = CacheEngineContext {
            engine_name: EngineName::CacheEngine,
            event_publisher,
            event_receiver: vec![exchange_event_receiver],
            cache: Arc::new(RwLock::new(HashMap::new())),
            command_publisher,
            command_receiver: Arc::new(Mutex::new(command_receiver)),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }

    pub async fn add_cache_key(&self, cache_key: CacheKey, max_size: Option<u32>, ttl: Duration) -> Result<(), String> {
        let mut context = self.context.write().await;
        let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
        cache_engine_context.add_cache_key(cache_key, max_size, ttl).await
    }

    pub async fn get_cache_key(&self, cache_key_type : Option<&str>) -> Result<Vec<String>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context.as_any().downcast_ref::<CacheEngineContext>().unwrap();
        // 获取hashmap所有的key
        let cache: tokio::sync::RwLockReadGuard<'_, HashMap<CacheKey, types::cache::CacheEntry>> = cache_engine_context.cache.read().await;
        if let Some(cache_key_type) = cache_key_type {
            match cache_key_type {
                "kline" => {
                    let keys = cache.keys().filter(|key| matches!(key, CacheKey::Kline(_))).map(|key: &CacheKey| key.get_key()).collect();
                    Ok(keys)
                },
                "indicator" => {
                    let keys = cache.keys().filter(|key| matches!(key, CacheKey::Indicator(_))).map(|key: &CacheKey| key.get_key()).collect();
                    Ok(keys)
                },
                _ => Err("Invalid cache key type".to_string()),
            }
        } else {
            let keys = cache.keys().map(|key: &CacheKey| key.get_key()).collect();
            Ok(keys)
        }
    }

    pub async fn get_cache_value(&self, cache_key: &CacheKey, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        let context = self.context.read().await;
        let cache_engine_context = context.as_any().downcast_ref::<CacheEngineContext>().unwrap();

        cache_engine_context.get_cache(cache_key, index, limit).await
    }

    pub async fn get_memory_size(&self) -> Result<HashMap<String, u32>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context.as_any().downcast_ref::<CacheEngineContext>().unwrap();
        let cache: tokio::sync::RwLockReadGuard<'_, HashMap<CacheKey, types::cache::CacheEntry>> = cache_engine_context.cache.read().await;
        let mut memory_size = HashMap::new();
        for (key, entry) in cache.iter() {
            memory_size.insert(key.get_key(), entry.get_memory_size());
        }
        Ok(memory_size)
    }


    pub async fn initialize_kline_cache(&self, exchange: Exchange, symbol: String, interval: KlineInterval, kline_series: Vec<Kline>) {
        let mut context = self.context.write().await;
        let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
        let cache_key = CacheKey::Kline(KlineCacheKey::new(exchange, symbol, interval));
        let cache_series = kline_series.into_iter().map(|kline| kline.into()).collect();
        cache_engine_context.initialize_cache(cache_key, cache_series).await;
    }

    pub async fn update_kline_cache(&self, exchange: Exchange, symbol: String, interval: KlineInterval, kline: Kline) {
        let mut context = self.context.write().await;
        let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
        let cache_key = CacheKey::Kline(KlineCacheKey::new(exchange, symbol, interval));
        cache_engine_context.update_cache(cache_key, kline.into()).await;
    }

    pub async fn initialize_indicator_cache(
        &self, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        indicator_config: IndicatorConfig,
        indicator_series: Vec<Indicator>) {
        let mut context = self.context.write().await;
        let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
        let cache_key = CacheKey::Indicator(IndicatorCacheKey::new(exchange, symbol, interval, indicator_config));
        let cache_series = indicator_series.into_iter().map(|indicator| indicator.into()).collect();
        cache_engine_context.initialize_cache(cache_key, cache_series).await;
    }

    pub async fn update_indicator_cache(
        &self, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        indicator_config: IndicatorConfig,
        indicator: Indicator) {
        let mut context = self.context.write().await;
        let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
        let cache_key = CacheKey::Indicator(IndicatorCacheKey::new(exchange, symbol, interval, indicator_config));
        cache_engine_context.update_cache(cache_key.clone(), indicator.into()).await;
    }
}