use std::collections::VecDeque;
use std::time::Duration;
use types::new_cache::CacheKey;
use types::new_cache::CacheValue;
use utils::get_utc8_timestamp_millis;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: CacheKey, // 缓存键
    pub data: VecDeque<CacheValue>, // 缓存数据
    pub create_time: i64, // 创建时间
    pub update_time: i64, // 更新时间
    pub max_size: u32, // 最大大小
    pub is_fresh: bool, // 是否新鲜
    pub ttl: Duration, // 过期时间 time to live

}

impl CacheEntry {
    pub fn new(key: CacheKey, max_size: u32, ttl: Duration) -> Self {
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

    // 初始化数据
    pub fn initialize(&mut self, data: VecDeque<CacheValue>) {
        self.data = data;
        self.is_fresh = true;
        self.update_time = get_utc8_timestamp_millis();
    }

    // 更新数据
    pub fn update(&mut self, cache_value: CacheValue) {
        // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k
        if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
            self.data.pop_back();
            self.data.push_back(cache_value);
        } else {
            // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
            // 如果缓存长度大于最大缓存长度，则删除最旧的一条数据
            if self.data.len() >= self.max_size as usize {
                self.data.pop_front();
            }
            self.data.push_back(cache_value);
        }
    }

    pub fn get_all_cache_value(&self) -> Vec<CacheValue> {
        self.data.iter().cloned().collect()
    }

    pub fn get_cache_value(&self, limit: u32) -> Vec<CacheValue> {
        // 如果limit大于等于数据长度，直接克隆并返回所有数据
        if limit as usize >= self.data.len() {
            return self.get_all_cache_value();
        }
        
        // 从后往前取limit条数据
        let start = self.data.len().saturating_sub(limit as usize);
        self.data.range(start..).cloned().collect()
    }

    pub fn get_timestamp_list(&self) -> Vec<i64> {
        self.data.iter().map(|value| value.get_timestamp()).collect()
    }

    pub fn get_cache_length(&self) -> usize {
        self.data.len()
    }
    
}