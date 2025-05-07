use event_center::Event;
use event_center::command_event::CommandEvent;
use event_center::command_event::cache_engine_command::CacheEngineCommand;
use event_center::exchange_event::ExchangeEvent;
use event_center::indicator_event::IndicatorEvent;
use types::cache::KlineCacheKey;
use types::market::Kline;
use std::fmt::Debug;
use types::indicator::IndicatorData;
use types::cache::IndicatorCacheKey;
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
            _ => {}
        }
    }
}

impl CacheEngineContext {
    async fn handle_exchange_event(&mut self, exchange_event: ExchangeEvent) {
        match exchange_event {
            ExchangeEvent::ExchangeKlineUpdate(event) => {
                tracing::debug!("处理交易所k线更新事件: {:?}", event.kline.close);

                //important: 需要注意锁的顺序，先获取读锁，再获取写锁
                // let event_publisher = self.event_publisher.clone();
                // let mut kline_cache_manager = self.kline_cache_manager.write().await;
                // kline_cache_manager.update_kline_cache(event, event_publisher.clone()).await;
            }
            ExchangeEvent::ExchangeKlineSeriesUpdate(event) => {
                tracing::debug!("处理交易所系列更新事件: {:?}", event);
                // let mut kline_cache_manager = self.kline_cache_manager.write().await;
                // kline_cache_manager.initialize_kline_series_cache(event).await;
            }
            _ => {}
        }
    }

    async fn handle_indicator_event(&mut self, indicator_event: IndicatorEvent) {
        tracing::info!("处理指标事件: {:?}", indicator_event);
    }
}
