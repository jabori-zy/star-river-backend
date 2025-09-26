pub mod cache_engine_context;
mod cache_entry;

use crate::Engine;
use crate::EngineContext;
use crate::EngineName;
use crate::cache_engine::cache_entry::CacheEntryTrait;
use async_trait::async_trait;
use cache_engine_context::CacheEngineContext;
use serde_json;
use star_river_core::key::Key;
use star_river_core::key::KeyTrait;
use star_river_core::market::QuantData;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub fn new() -> Self {
        let context = CacheEngineContext {
            engine_name: EngineName::CacheEngine,
            kline_cache: Arc::new(RwLock::new(HashMap::new())),
            kline_key_subscribe: Arc::new(RwLock::new(HashMap::new())),
            indicator_cache: Arc::new(RwLock::new(HashMap::new())),
            indicator_key_subscribe: Arc::new(RwLock::new(HashMap::new())),
        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        }
    }

    // pub async fn add_key(&self, key: Key, max_size: Option<u32>, ttl: Duration) -> Result<(), String> {
    //     let mut context = self.context.write().await;
    //     let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
    //     cache_engine_context.add_key(key, max_size, ttl).await
    // }

    pub async fn get_key(&self, key_type: Option<&str>) -> Result<Vec<String>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context.as_any().downcast_ref::<CacheEngineContext>().unwrap();

        if let Some(key_type) = key_type {
            match key_type {
                "kline" => {
                    let kline_cache = cache_engine_context.kline_cache.read().await;
                    let keys = kline_cache.keys().map(|key| key.get_key_str()).collect();
                    Ok(keys)
                }
                "indicator" => {
                    let indicator_cache = cache_engine_context.indicator_cache.read().await;
                    let keys = indicator_cache.keys().map(|key| key.get_key_str()).collect();
                    Ok(keys)
                }
                _ => Err("Invalid cache key type".to_string()),
            }
        } else {
            // 获取所有类型的key
            let kline_cache = cache_engine_context.kline_cache.read().await;
            let indicator_cache = cache_engine_context.indicator_cache.read().await;

            let mut keys = Vec::new();
            keys.extend(kline_cache.keys().map(|key| key.get_key_str()));
            keys.extend(indicator_cache.keys().map(|key| key.get_key_str()));

            Ok(keys)
        }
    }

    // 获取缓存值
    pub async fn get_cache_value(
        &self,
        key: &Key,
        index: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context.as_any().downcast_ref::<CacheEngineContext>().unwrap();

        match key {
            Key::Kline(kline_key) => match cache_engine_context.get_kline_cache(kline_key, index, limit).await {
                Ok(klines) => {
                    let value: Vec<serde_json::Value> = klines.iter().map(|v| v.to_json()).collect();
                    Ok(value)
                }
                Err(e) => Err(format!("获取K线缓存失败: {}", e)),
            },
            Key::Indicator(indicator_key) => {
                match cache_engine_context
                    .get_indicator_cache(indicator_key, index, limit)
                    .await
                {
                    Ok(indicators) => {
                        let value: Vec<serde_json::Value> = indicators.iter().map(|v| v.to_json()).collect();
                        Ok(value)
                    }
                    Err(e) => Err(format!("获取指标缓存失败: {}", e)),
                }
            }
        }
    }

    pub async fn get_memory_size(&self) -> Result<HashMap<String, u32>, String> {
        let context = self.context.read().await;
        let cache_engine_context = context.as_any().downcast_ref::<CacheEngineContext>().unwrap();

        let mut memory_size = HashMap::new();

        // 获取K线缓存内存大小
        {
            let kline_cache = cache_engine_context.kline_cache.read().await;
            for (key, entry) in kline_cache.iter() {
                memory_size.insert(key.get_key_str(), entry.get_memory_size());
            }
        }

        // 获取指标缓存内存大小
        {
            let indicator_cache = cache_engine_context.indicator_cache.read().await;
            for (key, entry) in indicator_cache.iter() {
                memory_size.insert(key.get_key_str(), entry.get_memory_size());
            }
        }

        Ok(memory_size)
    }

    // pub async fn initialize_kline_cache(
    //     &self,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     start_time: Option<String>,
    //     end_time: Option<String>,
    //     kline_series: Vec<Kline>,
    // ) {
    //     let mut context = self.context.write().await;
    //     let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
    //     let key = Key::Kline(KlineKey::new(exchange, symbol, interval, start_time, end_time));
    //     let cache_series = kline_series.into_iter().map(|kline| kline.into()).collect();
    //     cache_engine_context.initialize_cache(key, cache_series).await;
    // }

    // pub async fn update_kline_cache(
    //     &self,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     start_time: Option<String>,
    //     end_time: Option<String>,
    //     kline: Kline,
    // ) {
    //     let mut context = self.context.write().await;
    //     let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
    //     let key = Key::Kline(KlineKey::new(exchange, symbol, interval, start_time, end_time));
    //     cache_engine_context.update_cache(key, kline.into()).await;
    // }

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
    // pub async fn initialize_indicator_cache(
    //     &self,
    //     kline_key: KlineKey,
    //     indicator_config: IndicatorConfig,
    //     indicator_series: Vec<Indicator>,
    // ) -> Key {
    //     let mut context = self.context.write().await;
    //     let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
    //     let key = Key::Indicator(IndicatorKey::new(kline_key, indicator_config));
    //     let cache_series = indicator_series.into_iter().map(|indicator| indicator.into()).collect();
    //     cache_engine_context.initialize_cache(key.clone(), cache_series).await;
    //     key
    // }

    // pub async fn update_indicator_cache(
    //     &self,
    //     kline_key: KlineKey,
    //     indicator_config: IndicatorConfig,
    //     indicator: Indicator,
    // ) {
    //     let mut context = self.context.write().await;
    //     let cache_engine_context = context.as_any_mut().downcast_mut::<CacheEngineContext>().unwrap();
    //     let key = Key::Indicator(IndicatorKey::new(kline_key, indicator_config));
    //     cache_engine_context.update_cache(key.clone(), indicator.into()).await;
    // }
}
