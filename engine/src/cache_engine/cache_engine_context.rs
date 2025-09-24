mod context_impl;

use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::communication::engine::cache_engine::CacheEngineCommand;
use event_center::communication::engine::cache_engine::*;
use event_center::communication::engine::{EngineCommand, EngineResponse};
use event_center::event::Event;
use event_center::event::{ExchangeEvent, IndicatorEvent};
use star_river_core::cache::CacheItem;
use star_river_core::cache::key::KlineKey;
use star_river_core::cache::{
    CacheEntry,
    cache_entry::{IndicatorCacheEntry, KlineCacheEntry},
};
use star_river_core::cache::{CacheValue, Key};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::instrument;

#[derive(Debug)]
pub struct CacheEngineContext {
    pub engine_name: EngineName,
    pub cache: Arc<RwLock<HashMap<Key, CacheEntry>>>,
    // pub event_publisher: EventPublisher,
    // pub event_receiver: Vec<EventReceiver>,
    // pub command_publisher: CommandPublisher,
    // pub command_receiver: Arc<Mutex<CommandReceiver>>,
}

impl Clone for CacheEngineContext {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            // event_publisher: self.event_publisher.clone(),
            // event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            engine_name: self.engine_name.clone(),
            // command_publisher: self.command_publisher.clone(),
            // command_receiver: self.command_receiver.clone(),
        }
    }
}

impl CacheEngineContext {
    async fn handle_exchange_event(&mut self, exchange_event: ExchangeEvent) {
        // match exchange_event {
        // ExchangeEvent::ExchangeKlineUpdate(event) => {
        //     // 更新cache_key对应的数据
        //     let cache_key = Key::BacktestKline(BacktestKlineKey::new(event.exchange, event.symbol, event.interval));
        //     // 更新缓存
        //     self.update_cache(cache_key, event.kline.into()).await;
        // }
        //
        // ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
        //     tracing::debug!("处理k线系列更新事件");
        //     // 更新cache_key对应的数据
        //     let cache_key = Key::BacktestKline(BacktestKlineKey::new(event.exchange, event.symbol, event.interval));
        //     let cache_series = event.kline_series.into_iter().map(|kline| kline.into()).collect();
        //     self.initialize_cache(cache_key, cache_series).await;
        // }
        // 历史k线更新
        // ExchangeEvent::ExchangeKlineHistoryUpdate(event) => {
        //     // 更新cache_key对应的数据
        //     let key = KlineKey::new(
        //         event.exchange,
        //         event.symbol,
        //         event.interval,
        //         Some(event.time_range.start_date.to_string()),
        //         Some(event.time_range.end_date.to_string()),
        //     )
        //     .into();
        //     tracing::debug!("更新历史k线缓存: {:?}", key);
        //     let cache_series = event.kline_history.into_iter().map(|kline| kline.into()).collect();
        //     self.initialize_cache(key, cache_series).await;
        // }
        // _ => {}
        // }
    }

    async fn handle_indicator_event(&mut self, indicator_event: IndicatorEvent) {
        tracing::info!("处理指标事件: {:?}", indicator_event);
    }

    // 获取缓存数据
    pub async fn get_cache(&self, key: &Key, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        let cache = self.cache.read().await;
        let cache_entry = cache.get(key);
        if cache_entry.is_none() {
            tracing::error!("缓存键不存在: {:?}。获取数据失败", key);
            return vec![];
        }
        cache_entry.unwrap().get_cache_data(index, limit)
    }

    async fn get_cache_length(&self, key: &Key) -> u32 {
        let cache = self.cache.read().await;
        match cache.get(&key) {
            Some(cache_entry) => cache_entry.get_length(),
            None => {
                tracing::error!("缓存键不存在: {:?}。获取缓存长度失败", key);
                0
            }
        }
    }

    // 获取多个缓存数据
    pub async fn get_cache_multi(
        &self,
        keys: &Vec<Key>,
        index: Option<u32>,
        limit: Option<u32>,
    ) -> HashMap<Key, Vec<Arc<CacheValue>>> {
        let cache = self.cache.read().await;
        let mut cache_data = HashMap::new();
        for key in keys {
            let cache_entry = cache.get(&key);
            if cache_entry.is_none() {
                tracing::warn!("缓存键不存在: {:?}", key);
                cache_data.insert(key.clone(), vec![]);
                continue;
            }
            cache_data.insert(key.clone(), cache_entry.unwrap().get_cache_data(index, limit));
        }
        cache_data
    }

    pub async fn add_key(&mut self, key: Key, max_size: Option<u32>, ttl: Duration) -> Result<(), String> {
        let is_contain = { self.cache.read().await.contains_key(&key) };

        // 如果缓存键已存在，则不插入
        if !is_contain {
            match key.clone() {
                Key::Kline(backtest_kline_cache_key) => {
                    let mut cache = self.cache.write().await;
                    let cache_entry = KlineCacheEntry::new(backtest_kline_cache_key.clone(), max_size, ttl);
                    cache.insert(key, cache_entry.into());
                }
                Key::Indicator(history_indicator_cache_key) => {
                    let mut cache = self.cache.write().await;
                    let cache_entry = IndicatorCacheEntry::new(history_indicator_cache_key.clone(), max_size, ttl);
                    cache.insert(key, cache_entry.into());
                } // Key::Indicator(indicator_cache_key) => {
                  //     let is_contain = {
                  //         self.cache.read().await.contains_key(&indicator_cache_key.clone().into())
                  //     };
                  //     // 如果缓存键已存在，则不插入
                  //     if !is_contain {
                  //         // 1. 判断需要计算的k线的是否存在
                  //         // 创建这个指标对应的k线缓存键
                  //         let kline_cache_key = Key::Kline(KlineKey::new(indicator_cache_key.exchange.clone(), indicator_cache_key.symbol.clone(), indicator_cache_key.interval.clone()));
                  //         // 判断是否存在
                  //         let is_contain = {
                  //             self.cache.read().await.contains_key(&kline_cache_key)
                  //         };
                  //         // 如果k线缓存不存在，则不插入,并报错
                  //         if !is_contain {
                  //             tracing::error!("计算指标缓存键的k线缓存不存在: {:?}", kline_cache_key);
                  //             return Err("k线缓存不存在".to_string());
                  //         }
                  //         // 2. 如果存在，则获取K线缓存的max_size
                  //         let max_size = {
                  //             self.cache.read().await.get(&kline_cache_key).unwrap().get_max_size()
                  //         };
                  //         // 3. 插入指标缓存键，最大max_size使用k线缓存的max_size
                  //         let cache_entry = IndicatorCacheEntry::new(indicator_cache_key.clone(), max_size, Duration::from_secs(10));
                  //         tracing::info!("插入指标缓存键: {:?}", indicator_cache_key);
                  //         let mut cache = self.cache.write().await;
                  //         cache.insert(indicator_cache_key.into(), cache_entry.into());
                  //     }
                  // }
            }
        }
        Ok(())
    }

    // #[instrument(skip(self, cache_series), fields(key=?key, cache_series_length=cache_series.len()))]
    pub async fn initialize_cache(&mut self, key: Key, cache_series: Vec<CacheValue>) {
        // 更新cache_key对应的数据
        tracing::info!(key=?key, cache_series_length=cache_series.len(), "initailize cache value");
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&key).unwrap();
        // 初始化数据
        cache_entry.initialize(cache_series);
    }

    pub async fn update_cache(&mut self, key: Key, cache_calue: CacheValue) {
        // 先检查键是否存在，释放锁
        let key_exists = { self.cache.read().await.contains_key(&key) };

        if !key_exists {
            // 如果缓存键不存在，先添加键
            self.add_key(key.clone(), None, Duration::from_secs(10)).await.unwrap();
        }

        // 重新获取锁并更新
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&key).unwrap();

        if !key_exists {
            // 判断cache_entry长度
            cache_entry.initialize(vec![cache_calue]);
        } else {
            if cache_entry.get_length() == 0 {
                cache_entry.initialize(vec![cache_calue]);
            } else {
                cache_entry.update(cache_calue);
            }
        }
    }

    pub async fn clear_cache(&mut self, key: Key) {
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&key);
        if cache_entry.is_some() {
            cache_entry.unwrap().clear();
        } else {
            return;
        }
    }
}
