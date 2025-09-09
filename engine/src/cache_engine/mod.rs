pub mod cache_engine_context;

use crate::Engine;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use cache_engine_context::CacheEngineContext;
use event_center::EventPublisher;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use types::cache::key::IndicatorKey;
use types::cache::key::KlineKey;
use types::cache::{CacheValue, Key};
use types::indicator::Indicator;
use types::indicator::IndicatorConfig;
use types::market::{Exchange, Kline, KlineInterval};

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
    pub fn new(// event_publisher: EventPublisher,
        // command_publisher: CommandPublisher,
        // command_receiver: CommandReceiver,
        // exchange_event_receiver: EventReceiver,
    ) -> Self {
        let context = CacheEngineContext {
            engine_name: EngineName::CacheEngine,
            // event_publisher,
            // event_receiver: vec![exchange_event_receiver],
            cache: Arc::new(RwLock::new(HashMap::new())),
            // command_publisher,
            // command_receiver: Arc::new(Mutex::new(command_receiver)),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }

    pub async fn add_key(
        &self,
        key: Key,
        max_size: Option<u32>,
        ttl: Duration,
    ) -> Result<(), String> {
        let mut context = self.context.write().await;
        let cache_engine_context = context
            .as_any_mut()
            .downcast_mut::<CacheEngineContext>()
            .unwrap();
        cache_engine_context.add_key(key, max_size, ttl).await
    }

    pub async fn get_key(&self, key_type: Option<&str>) -> Result<Vec<String>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context
            .as_any()
            .downcast_ref::<CacheEngineContext>()
            .unwrap();
        // 获取hashmap所有的key
        let cache = cache_engine_context.cache.read().await;
        if let Some(key_type) = key_type {
            match key_type {
                "kline" => {
                    let keys = cache
                        .keys()
                        .filter(|key| matches!(key, Key::Kline(_)))
                        .map(|key: &Key| key.get_key())
                        .collect();
                    Ok(keys)
                }
                "indicator" => {
                    let keys = cache
                        .keys()
                        .filter(|key| matches!(key, Key::Indicator(_)))
                        .map(|key: &Key| key.get_key())
                        .collect();
                    Ok(keys)
                }
                _ => Err("Invalid cache key type".to_string()),
            }
        } else {
            let keys = cache.keys().map(|key: &Key| key.get_key()).collect();
            Ok(keys)
        }
    }

    // 获取缓存值
    pub async fn get_cache_value(
        &self,
        key: &Key,
        index: Option<u32>,
        limit: Option<u32>,
    ) -> Vec<Arc<CacheValue>> {
        let context = self.context.read().await;
        let cache_engine_context = context
            .as_any()
            .downcast_ref::<CacheEngineContext>()
            .unwrap();

        cache_engine_context.get_cache(key, index, limit).await
    }

    pub async fn get_memory_size(&self) -> Result<HashMap<String, u32>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context
            .as_any()
            .downcast_ref::<CacheEngineContext>()
            .unwrap();
        let cache: tokio::sync::RwLockReadGuard<'_, HashMap<Key, types::cache::CacheEntry>> =
            cache_engine_context.cache.read().await;
        let mut memory_size = HashMap::new();
        for (key, entry) in cache.iter() {
            memory_size.insert(key.get_key(), entry.get_memory_size());
        }
        Ok(memory_size)
    }

    pub async fn initialize_kline_cache(
        &self,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        start_time: Option<String>,
        end_time: Option<String>,
        kline_series: Vec<Kline>,
    ) {
        let mut context = self.context.write().await;
        let cache_engine_context = context
            .as_any_mut()
            .downcast_mut::<CacheEngineContext>()
            .unwrap();
        let key = Key::Kline(KlineKey::new(
            exchange, symbol, interval, start_time, end_time,
        ));
        let cache_series = kline_series.into_iter().map(|kline| kline.into()).collect();
        cache_engine_context
            .initialize_cache(key, cache_series)
            .await;
    }

    pub async fn update_kline_cache(
        &self,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        start_time: Option<String>,
        end_time: Option<String>,
        kline: Kline,
    ) {
        let mut context = self.context.write().await;
        let cache_engine_context = context
            .as_any_mut()
            .downcast_mut::<CacheEngineContext>()
            .unwrap();
        let key = Key::Kline(KlineKey::new(
            exchange, symbol, interval, start_time, end_time,
        ));
        cache_engine_context.update_cache(key, kline.into()).await;
    }

    // pub async fn initialize_indicator_cache(
    //     &self,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     indicator_config: IndicatorConfig,
    //     indicator_series: Vec<Indicator>) {
    //     let mut context = self.context.write().await;
    //     let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
    //     let cache_key = Key::BacktestIndicator(BacktestIndicatorKey::new(exchange, symbol, interval, indicator_config));
    //     let cache_series = indicator_series.into_iter().map(|indicator| indicator.into()).collect();
    //     cache_engine_context.initialize_cache(cache_key, cache_series).await;
    // }

    // 初始化回测指标缓存
    pub async fn initialize_indicator_cache(
        &self,
        kline_key: KlineKey,
        indicator_config: IndicatorConfig,
        indicator_series: Vec<Indicator>,
    ) -> Key {
        let mut context = self.context.write().await;
        let cache_engine_context = context
            .as_any_mut()
            .downcast_mut::<CacheEngineContext>()
            .unwrap();
        let key = Key::Indicator(IndicatorKey::new(kline_key, indicator_config));
        let cache_series = indicator_series
            .into_iter()
            .map(|indicator| indicator.into())
            .collect();
        cache_engine_context
            .initialize_cache(key.clone(), cache_series)
            .await;
        key
    }

    pub async fn update_indicator_cache(
        &self,
        kline_key: KlineKey,
        indicator_config: IndicatorConfig,
        indicator: Indicator,
    ) {
        let mut context = self.context.write().await;
        let cache_engine_context = context
            .as_any_mut()
            .downcast_mut::<CacheEngineContext>()
            .unwrap();
        let key = Key::Indicator(IndicatorKey::new(kline_key, indicator_config));
        cache_engine_context
            .update_cache(key.clone(), indicator.into())
            .await;
    }
}
