mod cache_engine_context;
mod kline_cache_manager;
mod indicator_cache_manager;

use utils::get_utc8_timestamp_millis;
use std::collections::HashMap;
use std::time::Duration;
use std::collections::VecDeque;
use std::fmt::Debug;
use types::cache::CacheKey;
use cache_engine_context::CacheEngineContext;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::EventPublisher;
use crate::Engine;
use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::Event;
use std::any::Any;


#[derive(Debug, Clone)]
pub struct CacheEntry<K: CacheKey, T> {
    pub key: K,
    pub batch_id: Option<String>,
    pub data: VecDeque<T>,
    pub created_at: i64,
    pub updated_at: i64,
    pub max_size: usize,
    pub is_fresh: bool, // 是否为新鲜数据
    pub ttl: Duration, // 数据缓存时间
}

impl<K: CacheKey, T: Debug> CacheEntry<K, T> {
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
pub struct CacheManager<K: CacheKey, T> {
    pub cache: HashMap<K, CacheEntry<K, T>>,
    // 缓存key有哪些策略共同使用
    pub subscribed_strategy: HashMap<K, Vec<i64>>,
    pub max_cache_size: usize
}



impl<K: CacheKey, T: Debug> CacheManager<K, T> {
    pub fn new() -> Self {
        Self { 
            cache: HashMap::new(), 
            subscribed_strategy: HashMap::new(), 
            max_cache_size: 1000, 
        }
    }

    // 添加订阅
    pub fn add_cache_key(&mut self, strategy_id: i64, cache_key: K) {
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
}


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
    pub fn new(
        event_publisher: EventPublisher,
        exchange_event_receiver: broadcast::Receiver<Event>,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let context = CacheEngineContext {
            engine_name: EngineName::CacheEngine,
            event_publisher,
            event_receiver: vec![exchange_event_receiver, response_event_receiver, request_event_receiver],
            kline_cache_manager: Arc::new(RwLock::new(CacheManager::new())),
            indicator_cache_manager: Arc::new(RwLock::new(CacheManager::new())),

        };
        Self {
            context: Arc::new(RwLock::new(Box::new(context)))
        }
    }
}

