pub mod kline_cache_manager;
pub mod indicator_cache_manager;

use utils::get_utc8_timestamp_millis;
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
use tokio::sync::RwLock;


// #[derive(Debug)]
// pub enum CacheData {
//     CacheKlineSeries(VecDeque<Kline>),
//     CacheIndicator,

// }

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



#[derive(Debug)]
pub struct CacheEngineState {
    pub kline_cache_manager: CacheManager<KlineCacheKey, Kline>,
    pub indicator_cache_manager: CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>>,
    pub event_publisher: EventPublisher,
}

#[derive(Debug)]
pub struct CacheEngine {
    // pub kline_cache_manager: Arc<Mutex<CacheManager<KlineCacheKey, Kline>>>,
    // pub indicator_cache_manager: Arc<Mutex<CacheManager<IndicatorCacheKey, Box<dyn IndicatorData>>>>,
    // event_center: Arc<Mutex<EventCenter>>,
    // event_publisher: EventPublisher,
    state: Arc<RwLock<CacheEngineState>>,
    exchange_event_receiver: broadcast::Receiver<Event>, 
    indicator_event_receiver: broadcast::Receiver<Event>,
    command_event_receiver: broadcast::Receiver<Event>,

}

impl Clone for CacheEngine {
    fn clone(&self) -> Self {
        Self {
            exchange_event_receiver: self.exchange_event_receiver.resubscribe(),
            indicator_event_receiver: self.indicator_event_receiver.resubscribe(),
            command_event_receiver: self.command_event_receiver.resubscribe(),
            state: self.state.clone(),
        }
    }
}


impl CacheEngine {
    pub fn new(
        exchange_event_receiver: broadcast::Receiver<Event>,
        indicator_event_receiver: broadcast::Receiver<Event>,
        command_event_receiver: broadcast::Receiver<Event>,
        event_publisher: EventPublisher, 
    ) -> Self {
        Self {
            exchange_event_receiver,
            indicator_event_receiver,
            command_event_receiver,
            state: Arc::new(RwLock::new(CacheEngineState {
                kline_cache_manager: CacheManager::new(),
                indicator_cache_manager: CacheManager::new(),
                event_publisher,
            })),
        }
    }

    async fn listen(&mut self,
        internal_tx: mpsc::Sender<Event>,
    ) {
        let mut exchange_rx = self.exchange_event_receiver.resubscribe();
        let mut command_rx = self.command_event_receiver.resubscribe();
        let mut indicator_rx = self.indicator_event_receiver.resubscribe();

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
    async fn handle_events(mut internal_rx: mpsc::Receiver<Event>, state: Arc<RwLock<CacheEngineState>>) {
        loop {
            let event = internal_rx.recv().await.unwrap();
            match event {
                Event::Command(command_event) => {
                    CacheEngine::handle_command_event(command_event, state.clone()).await;
                }
                Event::Exchange(exchange_event) => {
                    CacheEngine::handle_exchange_event(exchange_event, state.clone()).await;
                }
                Event::Indicator(indicator_event) => {
                    CacheEngine::handle_indicator_event(indicator_event, state.clone()).await;
                }
                _ => {}
            }
        }
    }

    async fn handle_command_event(command_event: CommandEvent, state: Arc<RwLock<CacheEngineState>>) {
        
        match command_event {
            // 处理k线缓存的命令
            CommandEvent::KlineCacheManager(command) => {
                match command {

                    KlineCacheManagerCommand::SubscribeKline(params) => {
                        let kline_cache_manager = &mut state.write().await.kline_cache_manager;
                        kline_cache_manager.subscribe(params.cache_key);
                    }
                }
            }
            // 处理指标缓存的命令
            CommandEvent::IndicatorCacheManager(command) => {
                match command {
                    IndicatorCacheManagerCommand::SubscribeIndicator(params) => {
                        let indicator_cache_manager = &mut state.write().await.indicator_cache_manager;
                        indicator_cache_manager.subscribe(params.cache_key);
                    }
                    IndicatorCacheManagerCommand::GetSubscribedIndicator(params) => {
                        let event_publisher = state.read().await.event_publisher.clone();
                        let indicator_cache_manager = &mut state.write().await.indicator_cache_manager;
                        indicator_cache_manager.get_subscribed_indicator(params, event_publisher);
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_exchange_event(exchange_event: ExchangeEvent, state: Arc<RwLock<CacheEngineState>>) {
        match exchange_event {
            ExchangeEvent::ExchangeKlineUpdate(event) => {
                // tracing::debug!("处理交易所k线更新事件: {:?}", event);

                //important: 需要注意锁的顺序，先获取读锁，再获取写锁
                let event_publisher = {
                    let state = state.read().await;
                    state.event_publisher.clone()
                };
                let kline_cache_manager = &mut state.write().await.kline_cache_manager;
                kline_cache_manager.update_kline_cache(event, event_publisher.clone()).await;
            }
            ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
                // tracing::debug!("处理交易所系列更新事件: {:?}", event);
                let kline_cache_manager = &mut state.write().await.kline_cache_manager;
                kline_cache_manager.initialize_kline_series_cache(event).await;
            }
            _ => {}
        }
    }

    async fn handle_indicator_event(indicator_event: IndicatorEvent, state: Arc<RwLock<CacheEngineState>>) {
        // tracing::info!("处理指标事件: {:?}", indicator_event);
    }
    
    pub async fn start(&mut self) 
    {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        
        self.listen(internal_tx).await;
        
        let state = self.state.clone();
        tokio::spawn(async move {
            CacheEngine::handle_events(internal_rx, state).await;   
        });

        tracing::info!("数据缓存引擎启动成功, 开始监听...");
    }

}
