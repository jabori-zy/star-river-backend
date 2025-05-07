use crate::cache_engine::cache_engine_type::{CacheManager, CacheEntry};

use event_center::exchange_event::{ExchangeKlineSeriesUpdateEvent, ExchangeKlineUpdateEvent};
use event_center::market_event::{MarketEvent, KlineSeriesInfo};
use types::market::Kline;
use std::collections::VecDeque;
use utils::get_utc8_timestamp_millis;
use types::market::KlineSeries;
use types::cache::KlineCacheKey;
use event_center::EventPublisher;


impl CacheEntry<KlineCacheKey, Kline> {
    pub fn initialize(&mut self, batch_id: String, data: VecDeque<Kline>) {
        self.batch_id = Some(batch_id);
        self.data = data;
        self.is_fresh = true;
        self.updated_at = get_utc8_timestamp_millis();
    }

    pub fn insert_or_update(&mut self, kline: Kline, batch_id: String) {
        // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
        if self.data.back().unwrap().timestamp == kline.timestamp {
            self.data.pop_back();
            self.data.push_back(kline);
            self.is_fresh = true;
            self.batch_id = Some(batch_id);
            self.updated_at = get_utc8_timestamp_millis();
        } else {
            self.data.push_back(kline);
            self.is_fresh = true;
            self.batch_id = Some(batch_id);
            self.updated_at = get_utc8_timestamp_millis();
        }

    }
}

impl From<CacheEntry<KlineCacheKey, Kline>> for KlineSeries {
    fn from(cache_entry: CacheEntry<KlineCacheKey, Kline>) -> Self {
        KlineSeries {
            exchange: cache_entry.key.exchange,
            symbol: cache_entry.key.symbol,
            interval: cache_entry.key.interval,
            series: cache_entry.data.into_iter().collect()
        }
    }
}

impl CacheManager<KlineCacheKey, Kline> {
    // 初始化k线缓存
    pub async fn initialize_kline_series_cache(&mut self, exchange_klineseries_event: ExchangeKlineSeriesUpdateEvent) {
        // 从事件中解析出CacheKey
        let exchange =exchange_klineseries_event.exchange;
        let symbol = exchange_klineseries_event.symbol;
        let interval = exchange_klineseries_event.interval;
        let cache_key = KlineCacheKey {
            exchange,
            symbol,
            interval,
        };
        // tracing::debug!("初始化k线缓存, cache_key: {:?}", cache_key);

        // 判断key是否已存在，如果不存在，则添加订阅
        // if !self.cache.contains_key(&cache_key) {
        //     self.add_cache_key(cache_key.clone());
        // }

        // 初始化缓存
        let kline_series = exchange_klineseries_event.kline_series;
        let batch_id = exchange_klineseries_event.batch_id;

        let cache_entry = self.cache.get_mut(&cache_key).unwrap();
        cache_entry.initialize(batch_id, kline_series.series.into_iter().collect());
        // tracing::debug!("初始化k线缓存成功, cache_entry: {:?}", cache_entry);

    }

    pub async fn update_kline_cache(&mut self, kline_update_event: ExchangeKlineUpdateEvent, event_publisher: EventPublisher) {    
        // tracing::debug!("更新k线缓存, kline_update_event: {:?}", kline_update_event);

        let exchange = kline_update_event.exchange;
        let symbol = kline_update_event.symbol;
        let interval = kline_update_event.interval;
        let cache_key = KlineCacheKey {
            exchange: exchange.clone(),
            symbol: symbol.clone(),
            interval: interval.clone(),
        };

        // 判断key是否已存在，如果不存在，则跳过
        if !self.cache.contains_key(&cache_key) {
            tracing::warn!("数据订阅不存在, cache_key: {:?}", cache_key);
            return;
        }

        let cache_entry: &mut CacheEntry<KlineCacheKey, Kline> = self.cache.get_mut(&cache_key).unwrap();
        let batch_id = kline_update_event.batch_id.clone();
        // 更新或者插入数据
        cache_entry.insert_or_update(kline_update_event.kline, batch_id.clone());
        // tracing::debug!("更新k线缓存成功, cache_entry: {:?}", cache_entry);

        // 发布事件
        self.publish_kline_series(event_publisher, cache_key, 50).await;
        
        
    }

    // 发布k线更新事件
    pub async fn publish_kline_series(&self, event_publisher: EventPublisher, cache_key: KlineCacheKey, limit: usize) {
        let cache_entry = self.cache.get(&cache_key).unwrap();
        // 从后往前取limit条数据
        let kline_series = cache_entry.data.iter().rev().take(limit).cloned().collect::<Vec<Kline>>().into_iter().rev().collect();
        let kline_series = KlineSeries {
            exchange: cache_key.exchange.clone(),
            symbol: cache_key.symbol.clone(),
            interval: cache_key.interval.clone(),
            series: kline_series,
        };
        let kline_series_update_event = MarketEvent::KlineSeriesUpdate(KlineSeriesInfo {
            exchange: cache_key.exchange,
            symbol: cache_key.symbol,
            interval: cache_key.interval,
            kline_series,
            event_timestamp: get_utc8_timestamp_millis(),
            batch_id: cache_entry.batch_id.as_ref().unwrap().clone(),
        }).into();
        event_publisher.publish(kline_series_update_event).unwrap();
    }
}