use crate::cache::Key;
use crate::cache::*;
use deepsize::DeepSizeOf;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use crate::utils::get_utc8_timestamp_millis;

#[derive(Debug, Clone)]
pub struct GenericCacheEntry<K: Clone + Debug + Into<Key>> {
    pub key: K,
    pub data: VecDeque<Arc<CacheValue>>,
    pub create_time: i64,
    pub update_time: i64,
    pub max_size: Option<u32>, // 如果为None，则表示无限缓存
    pub is_fresh: bool,
    pub ttl: Duration,
}

impl<K: Clone + Debug + Into<Key>> CacheEntryTrait for GenericCacheEntry<K> {
    fn initialize(&mut self, data: Vec<CacheValue>) {
        self.data = data.into_iter().map(|value| value.into()).collect();
        self.is_fresh = true;
        self.update_time = get_utc8_timestamp_millis();
    }

    fn update(&mut self, cache_value: CacheValue) {
        // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
        if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
            self.data.pop_back();
            self.data.push_back(cache_value.into());
        } else {
            // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
            // 如果max_size为None，则无限缓存
            if self.max_size.is_none() {
                self.data.push_back(cache_value.into());
            } else {
                // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
                if self.data.len() >= self.max_size.unwrap() as usize {
                    self.data.pop_front();
                }
                self.data.push_back(cache_value.into());
            }
        }
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn get_key(&self) -> Key {
        self.key.clone().into()
    }

    fn get_all_cache_data(&self) -> Vec<Arc<CacheValue>> {
        self.data.iter().cloned().collect()
    }

    fn get_cache_data(&self, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        match (index, limit) {
            // 有index，有limit
            (Some(idx), Some(limit_val)) => {
                // 如果索引超出范围，返回空
                if idx as usize >= self.data.len() {
                    return Vec::new();
                }

                // 计算从索引开始向前取limit个元素
                let end = idx as usize + 1;
                let start = if limit_val as usize >= end {
                    0
                } else {
                    end - limit_val as usize
                };

                self.data.range(start..end).cloned().collect()
            }

            // 有index，无limit
            (Some(idx), None) => {
                // 如果索引超出范围，返回空
                if idx as usize >= self.data.len() {
                    return Vec::new();
                }

                // 从索引开始向前取所有元素（到开头）
                let end = idx as usize + 1;
                self.data.range(0..end).cloned().collect()
            }

            // 无index，有limit
            (None, Some(limit_val)) => {
                // 从后往前取limit条数据
                if limit_val as usize >= self.data.len() {
                    return self.get_all_cache_data();
                }

                let start = self.data.len().saturating_sub(limit_val as usize);
                self.data.range(start..).cloned().collect()
            }

            // 无index，无limit
            (None, None) => {
                // 如果limit和index都为None，则返回所有数据
                self.get_all_cache_data()
            }
        }
    }

    fn get_create_time(&self) -> i64 {
        self.create_time
    }

    fn get_update_time(&self) -> i64 {
        self.update_time
    }

    fn get_max_size(&self) -> u32 {
        if self.max_size.is_none() {
            return u32::MAX;
        }
        self.max_size.unwrap()
    }

    fn get_is_fresh(&self) -> bool {
        self.is_fresh
    }

    fn get_ttl(&self) -> Duration {
        self.ttl
    }

    fn get_length(&self) -> u32 {
        self.data.len() as u32
    }

    fn get_memory_size(&self) -> u32 {
        self.data
            .iter()
            .map(|value| value.deep_size_of() as u32)
            .sum()
    }
}

pub type KlineCacheEntry = GenericCacheEntry<KlineKey>;

impl KlineCacheEntry {
    pub fn new(key: KlineKey, max_size: Option<u32>, ttl: Duration) -> Self {
        Self {
            key,
            data: VecDeque::new(),
            create_time: get_utc8_timestamp_millis(),
            update_time: get_utc8_timestamp_millis(),
            max_size,
            is_fresh: false,
            ttl,
        }
    }
}

impl From<KlineCacheEntry> for CacheEntry {
    fn from(entry: KlineCacheEntry) -> Self {
        CacheEntry::Kline(entry)
    }
}

pub type IndicatorCacheEntry = GenericCacheEntry<IndicatorKey>;

impl From<IndicatorCacheEntry> for CacheEntry {
    fn from(entry: IndicatorCacheEntry) -> Self {
        CacheEntry::Indicator(entry)
    }
}

impl IndicatorCacheEntry {
    pub fn new(key: IndicatorKey, max_size: Option<u32>, ttl: Duration) -> Self {
        Self {
            key,
            data: VecDeque::new(),
            create_time: get_utc8_timestamp_millis(),
            update_time: get_utc8_timestamp_millis(),
            max_size,
            is_fresh: false,
            ttl,
        }
    }
}
