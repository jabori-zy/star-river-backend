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
use event_center::indicator_event::IndicatorEvent;

use std::collections::VecDeque;
use types::cache::KlineCacheKey;
use types::market::Kline;
use std::fmt::Debug;
use types::indicator::IndicatorData;
use types::cache::CacheKey;
use types::cache::IndicatorCacheKey;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use event_center::EventPublisher;


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
    event_publisher: EventPublisher,
}


impl CacheEngine {
    pub fn new(
        // event_center: Arc<Mutex<EventCenter>>, 
        event_publisher: EventPublisher, 
    ) -> Self {
        Self {
            kline_cache_manager: Arc::new(Mutex::new(CacheManager::new())),
            indicator_cache_manager: Arc::new(Mutex::new(CacheManager::new())),
            event_publisher,
        }
    }

    async fn listen(&mut self,
        mut exchange_rx: broadcast::Receiver<Event>,
        mut command_rx: broadcast::Receiver<Event>,
        mut indicator_rx: broadcast::Receiver<Event>,
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
                    Ok(event) = indicator_rx.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
    }

    // 处理接收到的事件
    async fn handle_events(&self, mut internal_rx: mpsc::Receiver<Event>) {
        loop {
            let event = internal_rx.recv().await.unwrap();
            match event {
                Event::Command(command_event) => {
                    self.handle_command_event(command_event).await;
                }
                Event::Exchange(exchange_event) => {
                    self.handle_exchange_event(exchange_event).await;
                }
                Event::Indicator(indicator_event) => {
                    self.handle_indicator_event(indicator_event).await;
                }
                _ => {}
            }
        }
    }

    async fn handle_command_event(&self, command_event: CommandEvent) {
        let mut kline_cache_manager = {
            let kline_cache_manager = self.kline_cache_manager.lock().await;
            kline_cache_manager
        };
        let mut indicator_cache_manager = {
            let indicator_cache_manager = self.indicator_cache_manager.lock().await;
            indicator_cache_manager
        };
        
        match command_event {
            // 处理k线缓存的命令
            CommandEvent::KlineCacheManager(command) => {
                match command {
                    KlineCacheManagerCommand::SubscribeKline(params) => {
                        kline_cache_manager.subscribe(params.cache_key);
                    }
                }
            }
            // 处理指标缓存的命令
            CommandEvent::IndicatorCacheManager(command) => {
                match command {
                    IndicatorCacheManagerCommand::SubscribeIndicator(params) => {
                        indicator_cache_manager.subscribe(params.cache_key);
                    }
                    IndicatorCacheManagerCommand::GetSubscribedIndicator(params) => {
                        indicator_cache_manager.get_subscribed_indicator(params, self.event_publisher.clone());
                    }
                }
            }
        }
    }

    async fn handle_exchange_event(&self, exchange_event: ExchangeEvent) {
        let mut kline_cache_manager = self.kline_cache_manager.lock().await;

        match exchange_event {
            ExchangeEvent::ExchangeKlineUpdate(event) => {
                kline_cache_manager.update_kline_cache(event, self.event_publisher.clone()).await;
            }
            ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
                kline_cache_manager.initialize_kline_series_cache(event).await;
            }
            _ => {}
        }
    }

    async fn handle_indicator_event(&self, indicator_event: IndicatorEvent) {
        // tracing::info!("处理指标事件: {:?}", indicator_event);
    }
    pub async fn start(
        &mut self, 
        exchange_event_receiver: broadcast::Receiver<Event>, 
        indicator_event_receiver: broadcast::Receiver<Event>,
        command_event_receiver: broadcast::Receiver<Event>,
    ) {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        
        self.listen(exchange_event_receiver, command_event_receiver, indicator_event_receiver, internal_tx).await;
        
        let cache_engine = self.clone();
        tokio::spawn(async move {
            cache_engine.handle_events(internal_rx).await;   
        });

        tracing::info!("数据缓存引擎启动成功, 开始监听...");
    }

}
