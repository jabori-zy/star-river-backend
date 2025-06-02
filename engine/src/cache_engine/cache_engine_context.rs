use event_center::Event;
use event_center::exchange_event::ExchangeEvent;
use event_center::indicator_event::IndicatorEvent;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::broadcast;
use event_center::EventPublisher;
use tokio::sync::RwLock;
use crate::EngineContext;
use crate::EngineName;
use std::any::Any;
use async_trait::async_trait;
use std::collections::HashMap;
use types::cache::{CacheKey, CacheValue};
use types::cache::cache_key::KlineCacheKey;
use event_center::command::Command;
use event_center::command::cache_engine_command::CacheEngineCommand;
use std::time::Duration;
use event_center::response::cache_engine_response::{GetCacheDataResponse, GetCacheDataMultiResponse, GetCacheLengthMultiResponse};
use chrono::Utc;
use event_center::response::Response;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::cache::{CacheEntry, cache_entry::{KlineCacheEntry, IndicatorCacheEntry, HistoryKlineCacheEntry, HistoryIndicatorCacheEntry}};
use event_center::{EventReceiver, CommandPublisher, CommandReceiver};
use tokio::sync::Mutex;
use event_center::response::cache_engine_response::AddCacheKeyResponse;
use types::cache::cache_key::BacktestKlineCacheKey;
use tracing::instrument;

#[derive(Debug)]
pub struct CacheEngineContext {
    pub engine_name: EngineName,
    pub cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<EventReceiver>,
    pub command_publisher: CommandPublisher,
    pub command_receiver: Arc<Mutex<CommandReceiver>>,
}

impl Clone for CacheEngineContext {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            engine_name: self.engine_name.clone(),
            command_publisher: self.command_publisher.clone(),
            command_receiver: self.command_receiver.clone(),
        }
    }
}

#[async_trait]
impl EngineContext for CacheEngineContext {
    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }


    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect()
    }

    fn get_command_publisher(&self) -> &CommandPublisher {
        &self.command_publisher
    }

    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>> {
        self.command_receiver.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Exchange(exchange_event) => {
                self.handle_exchange_event(exchange_event).await;
            }
            Event::Indicator(indicator_event) => {
                self.handle_indicator_event(indicator_event).await;
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::CacheEngine(command) => {
                match command {
                    // 添加缓存
                    CacheEngineCommand::AddCacheKey(params) => {
                        self.add_cache_key(params.cache_key.clone(), params.max_size, params.duration).await.unwrap();
                        let response = AddCacheKeyResponse {
                            code: 0,
                            message: "success".to_string(),
                            cache_key: params.cache_key,
                            response_timestamp: Utc::now().timestamp(),
                        };
                        let response_event = Response::CacheEngine(CacheEngineResponse::AddCacheKey(response));

                        params.responder.send(response_event.into()).unwrap();
                    }
                    
                    // 处理获取缓存数据命令
                    CacheEngineCommand::GetCache(params) => {
                        let data = self.get_cache(&params.cache_key, params.index, params.limit).await;
                        let response = GetCacheDataResponse {
                            code: 0,
                            message: "success".to_string(),
                            cache_key: params.cache_key,
                            cache_data: data,
                            response_timestamp: Utc::now().timestamp(),
                        };
                        let response = CacheEngineResponse::GetCacheData(response);
                        params.responder.send(response.into()).unwrap();
                    }
                    CacheEngineCommand::GetCacheMulti(params) => {
                        let multi_data = self.get_cache_multi(&params.cache_keys, params.index, params.limit).await;
                        let response = GetCacheDataMultiResponse {
                            code: 0,
                            message: "success".to_string(),
                            cache_data: multi_data.into_iter().map(|(cache_key, data)| (cache_key.get_key(), data.into_iter().map(|cache_value| cache_value.to_list()).collect())).collect(),
                            response_timestamp: Utc::now().timestamp()
                        };
                        let response = CacheEngineResponse::GetCacheDataMulti(response);
                        params.responder.send(response.into()).unwrap();
                    }
                    CacheEngineCommand::GetCacheLengthMulti(params) => {
                        let mut length_result = HashMap::new();
                        for cache_key in params.cache_keys.iter() {
                            length_result.insert(cache_key.clone(), self.get_cache_length(cache_key).await);
                        }

                        let get_cache_length_multi_response = GetCacheLengthMultiResponse {
                            code: 0,
                            message: "success".to_string(),
                            cache_length: length_result.clone(),
                            response_timestamp: Utc::now().timestamp()
                        };
                        tracing::debug!(cache_lengths = ?length_result, "get cache length multi");
                        let response = CacheEngineResponse::GetCacheLengthMulti(get_cache_length_multi_response);
                        params.responder.send(response.into()).unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl CacheEngineContext {
    async fn handle_exchange_event(&mut self, exchange_event: ExchangeEvent) {
        match exchange_event {
            ExchangeEvent::ExchangeKlineUpdate(event) => {
                // 更新cache_key对应的数据
                let cache_key = CacheKey::Kline(KlineCacheKey::new(event.exchange, event.symbol, event.interval));
                // 更新缓存
                self.update_cache(cache_key, event.kline.into()).await;
            }
            
            ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
                tracing::debug!("处理k线系列更新事件");
                // 更新cache_key对应的数据
                let cache_key = CacheKey::Kline(KlineCacheKey::new(event.exchange, event.symbol, event.interval));
                let cache_series = event.kline_series.into_iter().map(|kline| kline.into()).collect();
                self.initialize_cache(cache_key, cache_series).await;
            }
            // 历史k线更新
            ExchangeEvent::ExchangeKlineHistoryUpdate(event) => {
                // 更新cache_key对应的数据
                let cache_key = BacktestKlineCacheKey::new(
                    event.exchange, 
                    event.symbol, 
                    event.interval,
                    event.time_range.start_date.to_string(), 
                    event.time_range.end_date.to_string())
                    .into();
                tracing::debug!("更新历史k线缓存: {:?}", cache_key);
                let cache_series = event.kline_history.into_iter().map(|kline| kline.into()).collect();
                self.initialize_cache(cache_key, cache_series).await;
            }
            _ => {}
        }
    }

    async fn handle_indicator_event(&mut self, indicator_event: IndicatorEvent) {
        tracing::info!("处理指标事件: {:?}", indicator_event);
    }

    // 获取缓存数据
    pub async fn get_cache(&self, cache_key: &CacheKey, index: Option<u32>, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&cache_key);
        if cache_entry.is_none() {
            tracing::error!("缓存键不存在: {:?}", cache_key);
            return vec![];
        }
        cache_entry.unwrap().get_cache_data(index, limit)

    }

    async fn get_cache_length(&self, cache_key: &CacheKey) -> u32 {
        let mut cache = self.cache.write().await;
        match cache.get_mut(&cache_key) {
            Some(cache_entry) => cache_entry.get_length(),
            None => {
                tracing::error!("缓存键不存在: {:?}", cache_key);
                0
            }
        }
    }

    // 获取多个缓存数据
    pub async fn get_cache_multi(&self, cache_keys: &Vec<CacheKey>, index: Option<u32>, limit: Option<u32>) -> HashMap<CacheKey, Vec<Arc<CacheValue>>> {
        let cache = self.cache.read().await;
        let mut cache_data = HashMap::new();
        for cache_key in cache_keys {
            let cache_entry = cache.get(&cache_key);
            if cache_entry.is_none() {
                tracing::warn!("缓存键不存在: {:?}", cache_key);
                cache_data.insert(cache_key.clone(), vec![]);
                continue;
            }
            cache_data.insert(cache_key.clone(), cache_entry.unwrap().get_cache_data(index, limit));
        }
        cache_data
    }

    pub async fn add_cache_key(&mut self, cache_key: CacheKey, max_size: Option<u32>, ttl: Duration) -> Result<(), String>{
        let is_contain = {
            self.cache.read().await.contains_key(&cache_key)
        };
        
        // 如果缓存键已存在，则不插入
        if !is_contain {
            match cache_key.clone() {
                CacheKey::Kline(kline_cache_key) => {
                    let mut cache = self.cache.write().await;
                    let cache_entry = KlineCacheEntry::new(kline_cache_key.clone(), max_size.unwrap_or(1000), ttl);
                    cache.insert(cache_key, cache_entry.into());
                }
                CacheKey::BacktestKline(backtest_kline_cache_key) => {
                    let mut cache = self.cache.write().await;
                    let cache_entry = HistoryKlineCacheEntry::new(backtest_kline_cache_key.clone(), max_size, ttl);
                    cache.insert(cache_key, cache_entry.into());
                }
                CacheKey::BacktestIndicator(history_indicator_cache_key) => {
                    let mut cache = self.cache.write().await;
                    let cache_entry = HistoryIndicatorCacheEntry::new(history_indicator_cache_key.clone(), max_size, ttl);
                    cache.insert(cache_key, cache_entry.into());
                }
                CacheKey::Indicator(indicator_cache_key) => {
                    let is_contain = {
                        self.cache.read().await.contains_key(&indicator_cache_key.clone().into())
                    };
                    // 如果缓存键已存在，则不插入
                    if !is_contain {
                        // 1. 判断需要计算的k线的是否存在
                        // 创建这个指标对应的k线缓存键
                        let kline_cache_key = CacheKey::Kline(KlineCacheKey::new(indicator_cache_key.exchange.clone(), indicator_cache_key.symbol.clone(), indicator_cache_key.interval.clone()));
                        // 判断是否存在
                        let is_contain = {
                            self.cache.read().await.contains_key(&kline_cache_key)
                        };
                        // 如果k线缓存不存在，则不插入,并报错
                        if !is_contain {
                            tracing::error!("计算指标缓存键的k线缓存不存在: {:?}", kline_cache_key);
                            return Err("k线缓存不存在".to_string());
                        }
                        // 2. 如果存在，则获取K线缓存的max_size
                        let max_size = {
                            self.cache.read().await.get(&kline_cache_key).unwrap().get_max_size()
                        };
                        // 3. 插入指标缓存键，最大max_size使用k线缓存的max_size
                        let cache_entry = IndicatorCacheEntry::new(indicator_cache_key.clone(), max_size, Duration::from_secs(10));
                        tracing::info!("插入指标缓存键: {:?}", indicator_cache_key);
                        let mut cache = self.cache.write().await;
                        cache.insert(indicator_cache_key.into(), cache_entry.into());
                    }
                }
            }
        }
        Ok(())
    }


    #[instrument(skip(self, cache_series), fields(cache_key=?cache_key, cache_series_length=cache_series.len()))]
    pub async fn initialize_cache(&mut self, cache_key: CacheKey, cache_series: Vec<CacheValue>) {
        // 更新cache_key对应的数据
        tracing::info!(cache_key=?cache_key, cache_series_length=cache_series.len(), "initailize cache value");
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&cache_key).unwrap();
        // 初始化数据
        cache_entry.initialize(cache_series);
    }

    pub async fn update_cache(&mut self, cache_key: CacheKey, cache_value: CacheValue) {
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&cache_key).unwrap();
        cache_entry.update(cache_value);
    }
}
