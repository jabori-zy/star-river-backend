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
use event_center::command_event::CommandEvent;
use event_center::command_event::cache_engine_command::CacheEngineCommand;
use std::time::Duration;
use event_center::response_event::cache_engine_response::GetCacheDataResponse;
use chrono::Utc;
use event_center::response_event::ResponseEvent;
use event_center::response_event::cache_engine_response::CacheEngineResponse;
use types::cache::{CacheEntry, cache_entry::{KlineCacheEntry, IndicatorCacheEntry}};

#[derive(Debug)]
pub struct CacheEngineContext {
    pub engine_name: EngineName,
    pub cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
}

impl Clone for CacheEngineContext {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            engine_name: self.engine_name.clone(),
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

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Exchange(exchange_event) => {
                self.handle_exchange_event(exchange_event).await;
            }
            Event::Indicator(indicator_event) => {
                self.handle_indicator_event(indicator_event).await;
            }
            Event::Command(command_event) => {
                self.handle_command_event(command_event).await;
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
                tracing::debug!("处理交易所系列更新事件: {:?}", event);
                // 更新cache_key对应的数据
                let cache_key = CacheKey::Kline(KlineCacheKey::new(event.exchange, event.symbol, event.interval));
                let cache_series = event.kline_series.into_iter().map(|kline| kline.into()).collect();
                self.initialize_cache(cache_key, cache_series).await;
            }
            _ => {}
        }
    }

    async fn handle_indicator_event(&mut self, indicator_event: IndicatorEvent) {
        tracing::info!("处理指标事件: {:?}", indicator_event);
    }

    async fn handle_command_event(&mut self, command_event: CommandEvent) {
        match command_event {
            CommandEvent::CacheEngine(command) => {
                match command {
                    // 添加缓存
                    CacheEngineCommand::AddCacheKey(params) => {
                        tracing::info!("接收到添加缓存键命令: {:?}", params);
                        self.add_cache_key(params.cache_key, params.max_size, params.duration).await.unwrap();
                        
                    }
                    
                    // 处理获取缓存数据命令
                    CacheEngineCommand::GetCache(params) => {
                        let data = self.get_cache(&params.cache_key, params.limit).await;
                        let response = GetCacheDataResponse {
                            code: 0,
                            message: "success".to_string(),
                            cache_key: params.cache_key,
                            cache_data: data.into_iter().collect(),
                            response_timestamp: Utc::now().timestamp(),
                            response_id: params.request_id, // 使用请求id
                        };
                        let response_event = ResponseEvent::CacheEngine(CacheEngineResponse::GetCacheData(response));
                        let _ = self.event_publisher.publish(response_event.into());
                    }
                }
            }
            _ => {}
        }
    }

    // 获取缓存数据
    pub async fn get_cache(&self, cache_key: &CacheKey, limit: Option<u32>) -> Vec<Arc<CacheValue>> {
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&cache_key).unwrap();
        cache_entry.get_cache_data(limit)

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
                CacheKey::Indicator(indicator_cache_key) => {
                    let is_contain = {
                        self.cache.read().await.contains_key(&indicator_cache_key.clone().into())
                    };
                    // 如果缓存键已存在，则不插入
                    if !is_contain {
                        // 1. 判断需要计算的k线的是否存在
                        // 创建这个指标对应的k线缓存键
                        let kline_cache_key = CacheKey::Kline(indicator_cache_key.kline_cache_key.clone());
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


    pub async fn initialize_cache(&mut self, cache_key: CacheKey, cache_series: Vec<CacheValue>) {
        // 更新cache_key对应的数据
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
