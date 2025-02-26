pub mod kline_cache_manager;
pub mod indicator_cache_manager;

use utils::get_utc8_timestamp;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::Event;
use std::time::Duration;
use event_center::command_event::{CommandEvent, KlineCacheManagerCommand, IndicatorCacheManagerCommand};
use event_center::exchange_event::ExchangeEvent;
use std::collections::VecDeque;
use types::cache::KlineCacheKey;
use types::market::Kline;
use std::fmt::Debug;
use types::indicator::IndicatorData;
use types::cache::CacheKey;
use types::cache::IndicatorCacheKey;
use tokio::sync::mpsc;
use tokio::sync::broadcast;

// #[derive(Debug)]
// pub enum CacheData {
//     CacheKlineSeries(VecDeque<Kline>),
//     CacheIndicator,

// }

#[derive(Debug, Clone)]
pub struct CacheEntry<K: CacheKey, T> {
    pub key: K,
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
            data: VecDeque::<T>::new(), 
            created_at: get_utc8_timestamp(), 
            updated_at: get_utc8_timestamp(), 
            max_size, 
            is_fresh: false, 
            ttl 
        }
    }
}


pub struct CacheManager<K: CacheKey, T> {
    pub cache: HashMap<K, CacheEntry<K, T>>,
    // pub event_center: Arc<Mutex<EventCenter>>,
    pub max_cache_size: usize
}



impl<K: CacheKey, T: Debug> CacheManager<K, T> {
    pub fn new() -> Self {
        Self { 
            cache: HashMap::new(), 
            // event_center, 
            max_cache_size: 1000, 
        }
    }

    // 添加订阅
    pub fn subscribe(&mut self, key: K) {
        // 如果key已经存在，则返回
        if self.cache.contains_key(&key) {
            tracing::warn!("缓存键已存在: {:?}", key);
            return;
        }


        let cache_entry = CacheEntry::new(key.clone(), self.max_cache_size, Duration::from_secs(60));
        self.cache.insert(key.clone(), cache_entry);

        
        tracing::debug!("添加订阅成功: {:?}", key);
    }

    pub fn remove_subscription(&mut self, key: K) {
        self.cache.remove(&key);
    }
}



#[derive(Clone)]
pub struct CacheEngine {
    pub kline_cache_manager: Arc<Mutex<CacheManager<KlineCacheKey, Kline>>>,
    pub indicator_cache_manager: Arc<Mutex<CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>>>>,
    // event_center: Arc<Mutex<EventCenter>>,
    event_publisher: broadcast::Sender<Event>,
}


impl CacheEngine {
    pub fn new(
        // event_center: Arc<Mutex<EventCenter>>, 
        event_publisher: broadcast::Sender<Event>, 
    ) -> Self {
        Self {
            kline_cache_manager: Arc::new(Mutex::new(CacheManager::new())),
            indicator_cache_manager: Arc::new(Mutex::new(CacheManager::new())),
            // event_center,
            event_publisher,
        }
    }

    async fn listen(&mut self,
        mut exchange_rx: broadcast::Receiver<Event>,
        mut command_rx: broadcast::Receiver<Event>,
        internal_tx: mpsc::Sender<Event>,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = command_rx.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                    Ok(event) = exchange_rx.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
    }

    // 处理接收到的事件
    async fn handle_events(internal_rx: &mut mpsc::Receiver<Event>, 
        kline_cache_manager: Arc<Mutex<CacheManager<KlineCacheKey, Kline>>>, 
        indicator_cache_manager: Arc<Mutex<CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>>>>,
        event_publisher: broadcast::Sender<Event>
    ) {
        
        loop {
            let event = internal_rx.recv().await.unwrap();
            match event {
                Event::Command(command_event) => {
                    CacheEngine::handle_command_event(command_event, kline_cache_manager.clone(), indicator_cache_manager.clone()).await;
                }
                Event::Exchange(exchange_event) => {
                    CacheEngine::handle_exchange_event(exchange_event, kline_cache_manager.clone(), event_publisher.clone()).await;
                }
                _ => {}
            }
        }
    }

    async fn handle_command_event(
        command_event: CommandEvent, 
        kline_cache_manager: Arc<Mutex<CacheManager<KlineCacheKey, Kline>>>, 
        indicator_cache_manager: Arc<Mutex<CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>>>>
    ) {

        let mut kline_cache_manager = kline_cache_manager.lock().await;
        let mut indicator_cache_manager = indicator_cache_manager.lock().await;
        
        match command_event {
            CommandEvent::KlineCacheManager(command) => {
                match command {
                    KlineCacheManagerCommand::SubscribeKline(params) => {
                        kline_cache_manager.subscribe(params.cache_key);
                    }
                }
            }
            CommandEvent::IndicatorCacheManager(command) => {
                match command {
                    IndicatorCacheManagerCommand::SubscribeIndicator(params) => {
                        indicator_cache_manager.subscribe(params.cache_key);
                    }
                }
            }
        }
    }

    async fn handle_exchange_event(
        exchange_event: ExchangeEvent, 
        kline_cache_manager: Arc<Mutex<CacheManager<KlineCacheKey, Kline>>>,
        event_publisher: broadcast::Sender<Event>
    ) 
    {
        
        let mut kline_cache_manager = kline_cache_manager.lock().await;

        match exchange_event {
            // 交易所单根k线更新
            ExchangeEvent::ExchangeKlineUpdate(event) => {
                kline_cache_manager.update_kline_cache(event, event_publisher).await;
            }
            // 交易所k线系列更新
            ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
                kline_cache_manager.initialize_kline_series_cache(event).await;
            }
            _ => {}
        }
    }

    pub async fn start(&mut self, exchange_event_receiver: broadcast::Receiver<Event>, command_event_receiver: broadcast::Receiver<Event>) {
        // 创建channel用于内部通信
        let (internal_tx, mut internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        

        // 接收外部消息，并在内部传递消息
        self.listen(exchange_event_receiver, command_event_receiver, internal_tx).await;
        

        let kline_cache_manager: Arc<Mutex<CacheManager<KlineCacheKey, Kline>>> = self.kline_cache_manager.clone();
        let indicator_cache_manager: Arc<Mutex<CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>>>> = self.indicator_cache_manager.clone();
        let event_publisher = self.event_publisher.clone();
        tokio::spawn(async move {
            CacheEngine::handle_events(&mut internal_rx, kline_cache_manager, indicator_cache_manager, event_publisher).await;   
        });

        tracing::info!("数据缓存引擎启动成功, 开始监听...");


    }

}
