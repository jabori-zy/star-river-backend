use event_center::Event;
use event_center::exchange_event::ExchangeEvent;
use event_center::indicator_event::IndicatorEvent;
use types::new_cache::KlineCacheKey;
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
use types::new_cache::CacheKey;
use crate::new_cache_engine::cache_engine_type::CacheEntry;
use event_center::command_event::CommandEvent;
use event_center::command_event::cache_engine_command::CacheEngineCommand;
use std::time::Duration;
use types::new_cache::CacheValueTrait;
use uuid::Uuid;
use event_center::response_event::cache_engine_response::GetCacheDataResponse;
use event_center::command_event::cache_engine_command::GetCacheDataParams;
use chrono::Utc;
use event_center::response_event::ResponseEvent;
use event_center::response_event::cache_engine_response::CacheEngineResponse;

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
                let mut cache = self.cache.write().await;
                let cache_entry = cache.get_mut(&cache_key).unwrap();
                // 插入或更新数据
                cache_entry.update(event.kline.to_cache_value());
                // tracing::debug!("当前缓存数据时间戳: {:?}， 当前缓存长度: {}， 最大缓存长度: {}", cache_entry.get_timestamp_list(), cache_entry.get_cache_length(), cache_entry.max_size);
            }
            ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
                tracing::debug!("处理交易所系列更新事件: {:?}", event);
                // 更新cache_key对应的数据
                let cache_key = CacheKey::Kline(KlineCacheKey::new(event.exchange, event.symbol, event.interval));
                let mut cache = self.cache.write().await;
                let cache_entry = cache.get_mut(&cache_key).unwrap();
                // 初始化数据
                cache_entry.initialize(event.kline_series.into_iter().map(|kline| kline.to_cache_value()).collect());
            }
            _ => {}
        }
    }

    async fn handle_indicator_event(&mut self, indicator_event: IndicatorEvent) {
        tracing::info!("处理指标事件: {:?}", indicator_event);
    }

    async fn handle_command_event(&mut self, command_event: CommandEvent) {
        match command_event {
            // 处理k线缓存的命令
            CommandEvent::CacheEngine(command) => {
                match command {
                    CacheEngineCommand::AddCacheKey(params) => {
                        tracing::info!("接收到添加k线缓存键命令: {:?}", params);
                        let mut cache = self.cache.write().await;
                        // 如果缓存键已存在，则不插入
                        if !cache.contains_key(&params.cache_key) {
                            let cache_entry = CacheEntry::new(params.cache_key.clone(), params.max_size, Duration::from_secs(10));
                            cache.insert(params.cache_key, cache_entry);
                        }
                    }
                    // 处理获取缓存数据命令
                    CacheEngineCommand::GetCacheData(params) => {
                        self.get_cache_data(params).await;
                    }
                    CacheEngineCommand::SubscribeIndicator(params) => {
                        // tracing::info!("接收到订阅指标命令: {:?}", params);
                        // let mut indicator_cache_manager = self.indicator_cache_manager.write().await;
                        // indicator_cache_manager.add_cache_key(params.cache_key, event_publisher);
                    }
                    _ => {}
                    // CacheEngineCommand::GetSubscribedIndicator(params) => {
                    //     tracing::info!("接收到获取订阅指标命令: {:?}", params);
                    //     let event_publisher = self.event_publisher.clone();
                    //     let indicator_cache_manager = self.indicator_cache_manager.write().await;
                    //     indicator_cache_manager.get_subscribed_indicator(params, event_publisher);
                    // }
                }
            }
            _ => {}
        }
    }

    async fn get_cache_data(&mut self, params: GetCacheDataParams) {
        tracing::info!("接收到获取k线数据命令: {:?}", params);
        let mut cache = self.cache.write().await;
        let cache_entry = cache.get_mut(&params.cache_key).unwrap();
        let data = cache_entry.get_cache_value(params.limit);
        let response = GetCacheDataResponse {
            cache_key: params.cache_key,
            cache_data: data.into_iter().collect(),
            response_timestamp: Utc::now().timestamp(),
            response_id: params.request_id, // 使用请求id
        };
        let response_event = ResponseEvent::CacheEngine(CacheEngineResponse::GetCacheData(response));
        let _ = self.event_publisher.publish(response_event.into());
    }
}
