use snafu::Report;
use star_river_core::indicator::Indicator;
use star_river_core::key::KeyTrait;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::market::{Kline, QuantData};
use std::collections::HashMap;

use super::{
    CacheEngineContext, CacheEngineError, CacheEntryTrait, Duration, IndicatorCacheEntry, Key, KeyNotFoundSnafu,
    KlineCacheEntry, StrategyId,
};

mod kline {
    use super::*;

    impl CacheEngineContext {
        // 添加K线缓存键
        pub async fn add_kline_key(
            &mut self,
            strategy_id: StrategyId,
            key: KlineKey,
            max_size: Option<u32>,
            ttl: Duration,
        ) {
            let mut kline_cache = self.kline_cache.write().await;
            let mut kline_key_subscribe = self.kline_key_subscribe.write().await;

            let subscribers = kline_key_subscribe.entry(key.clone()).or_insert_with(Vec::new);
            if !subscribers.contains(&strategy_id) {
                subscribers.push(strategy_id);
            }

            let key_clone = key.clone();
            kline_cache
                .entry(key)
                .or_insert_with(|| KlineCacheEntry::new(key_clone, max_size, ttl));
        }

        // 获取K线缓存数据
        pub async fn get_kline_cache(
            &self,
            key: &KlineKey,
            index: Option<u32>,
            limit: Option<u32>,
        ) -> Result<Vec<Kline>, CacheEngineError> {
            let kline_cache = self.kline_cache.read().await;
            match kline_cache.get(key) {
                Some(cache_entry) => Ok(cache_entry.get_data(index, limit)),
                None => {
                    let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                    let report = Report::from_error(&error);
                    tracing::error!("{}", report);
                    Err(error)
                }
            }
        }

        // 获取多个K线缓存数据
        pub async fn get_kline_cache_multi(
            &self,
            keys: &Vec<KlineKey>,
            index: Option<u32>,
            limit: Option<u32>,
        ) -> Result<HashMap<KlineKey, Vec<Kline>>, CacheEngineError> {
            let kline_cache = self.kline_cache.read().await;
            let mut cache_data = HashMap::new();

            for key in keys {
                match kline_cache.get(key) {
                    Some(cache_entry) => {
                        cache_data.insert(key.clone(), cache_entry.get_data(index, limit));
                    }
                    None => {
                        let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                        let report = Report::from_error(&error);
                        tracing::error!("{}", report);
                        return Err(error);
                    }
                }
            }
            Ok(cache_data)
        }

        // 获取K线缓存长度
        pub async fn get_kline_cache_length(&self, key: &KlineKey) -> Result<u32, CacheEngineError> {
            let kline_cache = self.kline_cache.read().await;
            match kline_cache.get(key) {
                Some(cache_entry) => Ok(cache_entry.get_length()),
                None => {
                    let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                    let report = Report::from_error(&error);
                    tracing::error!("{}", report);
                    Err(error)
                }
            }
        }

        // 获取多个K线缓存长度
        pub async fn get_kline_cache_length_multi(
            &self,
            keys: &Vec<KlineKey>,
        ) -> Result<HashMap<KlineKey, u32>, CacheEngineError> {
            let kline_cache = self.kline_cache.read().await;
            let mut cache_data = HashMap::new();

            for key in keys {
                match kline_cache.get(key) {
                    Some(cache_entry) => {
                        cache_data.insert(key.clone(), cache_entry.get_length());
                    }
                    None => {
                        let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                        let report = Report::from_error(&error);
                        tracing::error!("{}", report);
                        return Err(error);
                    }
                }
            }
            Ok(cache_data)
        }

        // 更新K线缓存数据
        pub async fn update_kline_cache(
            &mut self,
            strategy_id: StrategyId,
            key: KlineKey,
            cache_value: Kline,
        ) -> Result<(), CacheEngineError> {
            // 先检查键是否存在
            let key_exists = { self.kline_cache.read().await.contains_key(&key) };

            if !key_exists {
                // 如果缓存键不存在，先添加键
                self.add_kline_key(strategy_id, key.clone(), None, Duration::from_secs(10))
                    .await;
            }

            // 重新获取锁并更新
            let mut kline_cache = self.kline_cache.write().await;
            if let Some(cache_entry) = kline_cache.get_mut(&key) {
                if !key_exists || cache_entry.get_length() == 0 {
                    // 如果缓存不存在或为空，初始化数据
                    cache_entry.initialize(vec![cache_value]);
                } else {
                    // 否则更新数据
                    cache_entry.update(cache_value);
                }
                Ok(())
            } else {
                // 理论上不应该到达这里，因为前面已经确保键存在
                let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                let report = Report::from_error(&error);
                tracing::error!("{}", report);
                Err(error)
            }
        }

        // 清空K线缓存数据
        pub async fn clear_kline_cache(&mut self, key: &KlineKey) -> Result<(), CacheEngineError> {
            let mut kline_cache = self.kline_cache.write().await;
            if let Some(cache_entry) = kline_cache.get_mut(key) {
                cache_entry.clear();
                Ok(())
            } else {
                let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                let report = Report::from_error(&error);
                tracing::error!("{}", report);
                Err(error)
            }
        }
    }
}

mod indicator {
    use super::*;

    impl CacheEngineContext {
        // 添加指标缓存键
        pub async fn add_indicator_key(
            &mut self,
            strategy_id: StrategyId,
            key: IndicatorKey,
            max_size: Option<u32>,
            ttl: Duration,
        ) {
            let mut indicator_cache = self.indicator_cache.write().await;
            let mut indicator_key_subscribe = self.indicator_key_subscribe.write().await;

            let subscribers = indicator_key_subscribe.entry(key.clone()).or_insert_with(Vec::new);
            if !subscribers.contains(&strategy_id) {
                subscribers.push(strategy_id);
            }

            let key_clone = key.clone();
            indicator_cache
                .entry(key)
                .or_insert_with(|| IndicatorCacheEntry::new(key_clone, max_size, ttl));
        }

        // 获取指标缓存数据
        pub async fn get_indicator_cache(
            &self,
            key: &IndicatorKey,
            index: Option<u32>,
            limit: Option<u32>,
        ) -> Result<Vec<Indicator>, CacheEngineError> {
            let indicator_cache = self.indicator_cache.read().await;
            match indicator_cache.get(key) {
                Some(cache_entry) => Ok(cache_entry.get_data(index, limit)),
                None => {
                    let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                    let report = Report::from_error(&error);
                    tracing::error!("{}", report);
                    Err(error)
                }
            }
        }

        // 获取多个指标缓存数据
        pub async fn get_indicator_cache_multi(
            &self,
            keys: &Vec<IndicatorKey>,
            index: Option<u32>,
            limit: Option<u32>,
        ) -> Result<HashMap<IndicatorKey, Vec<Indicator>>, CacheEngineError> {
            let indicator_cache = self.indicator_cache.read().await;
            let mut cache_data = HashMap::new();

            for key in keys {
                match indicator_cache.get(key) {
                    Some(cache_entry) => {
                        cache_data.insert(key.clone(), cache_entry.get_data(index, limit));
                    }
                    None => {
                        let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                        let report = Report::from_error(&error);
                        tracing::error!("{}", report);
                        return Err(error);
                    }
                }
            }
            Ok(cache_data)
        }

        // 获取指标缓存长度
        pub async fn get_indicator_cache_length(&self, key: &IndicatorKey) -> Result<u32, CacheEngineError> {
            let indicator_cache = self.indicator_cache.read().await;
            match indicator_cache.get(key) {
                Some(cache_entry) => Ok(cache_entry.get_length()),
                None => {
                    let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                    let report = Report::from_error(&error);
                    tracing::error!("{}", report);
                    Err(error)
                }
            }
        }

        // 获取多个指标缓存长度
        pub async fn get_indicator_cache_length_multi(
            &self,
            keys: &Vec<IndicatorKey>,
        ) -> Result<HashMap<IndicatorKey, u32>, CacheEngineError> {
            let indicator_cache = self.indicator_cache.read().await;
            let mut cache_data = HashMap::new();
            for key in keys {
                match indicator_cache.get(key) {
                    Some(cache_entry) => {
                        cache_data.insert(key.clone(), cache_entry.get_length());
                    }
                    None => {
                        let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                        let report = Report::from_error(&error);
                        tracing::error!("{}", report);
                        return Err(error);
                    }
                }
            }
            Ok(cache_data)
        }

        // 更新指标缓存数据
        pub async fn update_indicator_cache(
            &mut self,
            strategy_id: StrategyId,
            key: IndicatorKey,
            cache_value: Indicator,
        ) -> Result<(), CacheEngineError> {
            // 先检查键是否存在
            let key_exists = { self.indicator_cache.read().await.contains_key(&key) };

            if !key_exists {
                // 如果缓存键不存在，先添加键
                self.add_indicator_key(strategy_id, key.clone(), None, Duration::from_secs(10))
                    .await;
            }

            // 重新获取锁并更新
            let mut indicator_cache = self.indicator_cache.write().await;
            if let Some(cache_entry) = indicator_cache.get_mut(&key) {
                if !key_exists || cache_entry.get_length() == 0 {
                    // 如果缓存不存在或为空，初始化数据
                    cache_entry.initialize(vec![cache_value]);
                } else {
                    // 否则更新数据
                    cache_entry.update(cache_value);
                }
                Ok(())
            } else {
                // 理论上不应该到达这里，因为前面已经确保键存在
                let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                let report = Report::from_error(&error);
                tracing::error!("{}", report);
                Err(error)
            }
        }

        // 清空指标缓存数据
        pub async fn clear_indicator_cache(&mut self, key: &IndicatorKey) -> Result<(), CacheEngineError> {
            let mut indicator_cache = self.indicator_cache.write().await;
            if let Some(cache_entry) = indicator_cache.get_mut(key) {
                cache_entry.clear();
                Ok(())
            } else {
                let error = KeyNotFoundSnafu { key: key.get_key_str() }.build();
                let report = Report::from_error(&error);
                tracing::error!("{}", report);
                Err(error)
            }
        }
    }
}
