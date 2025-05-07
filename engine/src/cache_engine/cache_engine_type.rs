use types::cache::CacheKey;
use std::collections::VecDeque;
use std::time::Duration;
use std::fmt::Debug;
use std::collections::HashMap;
use utils::get_utc8_timestamp_millis;
use types::custom_type::StrategyId;




#[derive(Debug, Clone)]
pub struct CacheEntry<K: CacheKey, T: Clone> {
    pub key: K,
    pub batch_id: Option<String>,
    pub data: VecDeque<T>,
    pub created_at: i64,
    pub updated_at: i64,
    pub max_size: usize,
    pub is_fresh: bool, // 是否为新鲜数据
    pub ttl: Duration, // 数据缓存时间
}

impl<K: CacheKey, T: Debug + Clone> CacheEntry<K, T> {
    pub fn new(key: K, max_size: usize, ttl: Duration) -> Self {
        Self { 
            key,
            batch_id: None,
            data: VecDeque::<T>::new(), 
            created_at: get_utc8_timestamp_millis(), 
            updated_at: get_utc8_timestamp_millis(),
            max_size, 
            is_fresh: false,
            ttl 
        }
    }
}


#[derive(Debug)]
pub struct CacheManager<K: CacheKey, T: Clone> {
    pub cache: HashMap<K, CacheEntry<K, T>>,
    // 缓存key有哪些策略共同使用
    pub subscribed_strategy: HashMap<K, Vec<StrategyId>>,
    pub max_cache_size: usize
}



impl<K: CacheKey, T: Debug + Clone> CacheManager<K, T> {
    pub fn new() -> Self {
        Self { 
            cache: HashMap::new(), 
            subscribed_strategy: HashMap::new(), 
            max_cache_size: 1000,
        }
    }

    // 添加订阅
    pub fn add_cache_key(&mut self, strategy_id: i32, cache_key: K) {
        // 如果key已经存在，则返回
        if self.cache.contains_key(&cache_key) {
            tracing::warn!("k线缓存键已存在: {:?}", cache_key);
            return;
        }


        let cache_entry = CacheEntry::new(cache_key.clone(), self.max_cache_size, Duration::from_secs(60));
        self.cache.insert(cache_key.clone(), cache_entry);

        // 添加订阅的策略id
        self.subscribed_strategy.entry(cache_key.clone()).or_insert(Vec::new()).push(strategy_id);
        tracing::debug!("添加k线缓存键成功: {:?}", cache_key);
    }

    pub fn remove_kline_cache_key(&mut self, key: K) {
        self.cache.remove(&key);
    }

    // 获取对应的cache_key的所有缓存数据
    pub fn get_all_cache_data(&self, cache_key: K) -> Vec<T> {
        self.cache.get(&cache_key).unwrap().data.clone().into_iter().collect()
    }

    // 获取对应的cache_key的最后一条缓存数据
    pub fn get_last_cache_data(&self, cache_key: K) -> Option<T> {
        self.cache.get(&cache_key).unwrap().data.back().cloned()
    }
}