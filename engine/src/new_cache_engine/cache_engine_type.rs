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
    pub max_size: usize, // 最大大小
    pub is_fresh: bool, // 是否新鲜
    pub ttl: Duration, // 过期时间 time to live

}

impl CacheEntry {
    pub fn new(key: CacheKey, max_size: usize, ttl: Duration) -> Self {
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

    pub fn initialize(&mut self, data: VecDeque<CacheValue>) {
        self.data = data;
        self.is_fresh = true;
        self.update_time = get_utc8_timestamp_millis();
    }

    pub fn insert_or_update(&mut self, cache_value: CacheValue) {
        if self.data.back().unwrap().get_timestamp() == cache_value.get_timestamp() {
            self.data.pop_back();
            self.data.push_back(cache_value);
        } else {
            self.data.push_back(cache_value);
        }
    }
    
}