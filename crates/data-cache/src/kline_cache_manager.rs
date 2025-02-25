use crate::{CacheManager, CacheEntry};
use event_center::market_event::{KlineSeriesEventInfo, ExchangeKlineEventInfo, ExchangeKlineSeriesEventInfo};
use types::market::Kline;
use std::collections::VecDeque;
use utils::get_utc8_timestamp;
use event_center::market_event::MarketEvent;
use types::market::KlineSeries;
use crate::KlineCacheKey;
use tokio::sync::broadcast;
use event_center::Event;


impl CacheEntry<KlineCacheKey, Kline> {
    pub fn initialize(&mut self, data: VecDeque<Kline>) {
        self.data = data;
        self.is_fresh = true;
        self.updated_at = get_utc8_timestamp();
    }

    pub fn insert_or_update(&mut self, kline: Kline) {
        // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
        if self.data.back().unwrap().timestamp == kline.timestamp {
            self.data.pop_back();
            self.data.push_back(kline);
            self.is_fresh = true;
            self.updated_at = get_utc8_timestamp();
        } else {
            self.data.push_back(kline);
            self.is_fresh = true;
            self.updated_at = get_utc8_timestamp();
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
    pub async fn initialize_kline_series_cache(&mut self, exchange_kline_series_event: ExchangeKlineSeriesEventInfo) {
        // 从事件中解析出CacheKey
        let exchange =exchange_kline_series_event.exchange;
        let symbol = exchange_kline_series_event.symbol;
        let interval = exchange_kline_series_event.interval;
        let cache_key = KlineCacheKey {
            exchange,
            symbol,
            interval,
        };
        tracing::debug!("初始化k线缓存, cache_key: {:?}", cache_key);

        // 判断key是否已存在，如果不存在，则添加订阅
        if !self.cache.contains_key(&cache_key) {
            self.subscribe(cache_key.clone());
        }

        // 初始化缓存
        let kline_series = exchange_kline_series_event.kline_series;

        let cache_entry = self.cache.get_mut(&cache_key).unwrap();
        cache_entry.initialize(kline_series.series.into_iter().collect());
        tracing::debug!("初始化k线缓存成功, cache_entry: {:?}", cache_entry);

    }

    pub async fn update_kline_cache(&mut self, kline_update_event: ExchangeKlineEventInfo, event_publisher: broadcast::Sender<Event>) {    
        tracing::debug!("更新k线缓存, kline_update_event: {:?}", kline_update_event);

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
        cache_entry.insert_or_update(kline_update_event.kline);
        tracing::debug!("更新k线缓存成功, cache_entry: {:?}", cache_entry);

        // 发布事件
        
        let kline_series_update_event = MarketEvent::KlineSeriesUpdate(KlineSeriesEventInfo {
            exchange,
            symbol,
            interval,
            kline_series: KlineSeries::from(cache_entry.clone()),
            event_timestamp: get_utc8_timestamp()
        }).into();
        tracing::info!("发布k线缓存事件: {:?}", kline_series_update_event);
        // let event_center: tokio::sync::MutexGuard<'_, event_center::EventCenter> = self.event_center.lock().await;
        // event_center.publish(kline_series_update_event).unwrap(); 
        let _ = event_publisher.send(kline_series_update_event);
        
        
    }
}