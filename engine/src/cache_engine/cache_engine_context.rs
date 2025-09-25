mod context_impl;
mod cache_operator;
mod command_handler;

use crate::EngineName;
use async_trait::async_trait;
use event_center::communication::engine::cache_engine::CacheEngineCommand;
use event_center::communication::engine::cache_engine::*;
use event_center::communication::engine::{EngineCommand, EngineResponse};
use event_center::event::Event;
use event_center::event::{ExchangeEvent, IndicatorEvent};
use star_river_core::key::key::{IndicatorKey, KlineKey};
use super::cache_entry::{IndicatorCacheEntry, KlineCacheEntry};
use star_river_core::key::Key;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::instrument;
use star_river_core::custom_type::StrategyId;
use crate::cache_engine::cache_entry::CacheEntryTrait;
use star_river_core::error::engine_error::cache_engine_error::*;

#[derive(Debug)]
pub struct CacheEngineContext {
    pub engine_name: EngineName,
    pub kline_cache: Arc<RwLock<HashMap<KlineKey, KlineCacheEntry>>>, // k线缓存
    pub kline_key_subscribe: Arc<RwLock<HashMap<KlineKey, Vec<StrategyId>>>>, // k线缓存订阅
    pub indicator_cache: Arc<RwLock<HashMap<IndicatorKey, IndicatorCacheEntry>>>, // 指标缓存
    pub indicator_key_subscribe: Arc<RwLock<HashMap<IndicatorKey, Vec<StrategyId>>>>, // 指标缓存订阅
}

impl Clone for CacheEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            kline_cache: self.kline_cache.clone(),
            kline_key_subscribe: self.kline_key_subscribe.clone(),
            indicator_cache: self.indicator_cache.clone(),
            indicator_key_subscribe: self.indicator_key_subscribe.clone(),
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

    // #[instrument(skip(self, cache_series), fields(key=?key, cache_series_length=cache_series.len()))]
    // pub async fn initialize_cache(&mut self, key: Key, cache_series: Vec<CacheValue>) {
    //     // 更新cache_key对应的数据
    //     tracing::info!(key=?key, cache_series_length=cache_series.len(), "initailize cache value");
    //     let mut cache = self.cache.write().await;
    //     let cache_entry = cache.get_mut(&key).unwrap();
    //     // 初始化数据
    //     cache_entry.initialize(cache_series);
    // }

    

    
}
