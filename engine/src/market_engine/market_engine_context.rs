use tokio::sync::broadcast;
use types::market::Exchange;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use std::any::Any;
use crate::EngineName;
use std::sync::Arc;
use event_center::command_event::CommandEvent;
use event_center::command_event::cache_engine_command::CacheEngineCommand;
use event_center::response_event::ResponseEvent;
use event_center::response_event::market_engine_response::{MarketEngineResponse, SubscribeKlineStreamSuccessResponse, UnsubscribeKlineStreamSuccessResponse};
use event_center::command_event::market_engine_command::{MarketEngineCommand, SubscribeKlineStreamParams, UnsubscribeKlineStreamParams};
use event_center::command_event::cache_engine_command::AddCacheKeyParams;
use types::new_cache::{CacheKey, KlineCacheKey};
use utils::get_utc8_timestamp_millis;
use types::market::KlineInterval;
use event_center::EventPublisher;
use tokio::sync::Mutex;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;





#[derive(Debug)]
pub struct MarketEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
}

impl Clone for MarketEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            exchange_engine: self.exchange_engine.clone()

        }
    }
}

#[async_trait]
impl EngineContext for MarketEngineContext {

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
            Event::Command(command_event) => {
                match command_event {
                    CommandEvent::MarketEngine(MarketEngineCommand::SubscribeKlineStream(params)) => {
                        self.subscribe_kline_stream(params).await.unwrap();
                    }
                    CommandEvent::MarketEngine(MarketEngineCommand::UnsubscribeKlineStream(params)) => {
                        self.unsubscribe_kline_stream(params).await.unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }

}

impl MarketEngineContext {

    fn add_cache_key(&self, strategy_id: i32, exchange: Exchange, symbol: String, interval: KlineInterval, max_size: u32) {
        // 调用缓存器的订阅事件
        let cache_key = CacheKey::Kline(KlineCacheKey {
            exchange: exchange,
            symbol: symbol.to_string(),
            interval: interval.clone(),
        });
        let params = AddCacheKeyParams {
            strategy_id,
            cache_key,
            max_size,
            sender: format!("strategy_{}", strategy_id),
            timestamp: get_utc8_timestamp_millis(),

        };
        let command = CacheEngineCommand::AddCacheKey(params);
        let command_event = CommandEvent::CacheEngine(command);

        self.get_event_publisher().publish(command_event.clone().into()).unwrap();
    }


    async fn subscribe_kline_stream(&self, params: SubscribeKlineStreamParams) -> Result<(), String> {
        // tracing::debug!("市场数据引擎订阅K线流: {:?}", params);
        // 添加缓存key
        self.add_cache_key(params.strategy_id, params.exchange.clone(), params.symbol.clone(), params.interval.clone(), params.cache_size);

        // 1. 先检查注册状态
        let is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(&params.account_id).await
        };

        if !is_registered {
            return Err(format!("交易所 {:?} 未注册", params.exchange));
        }

        // 2. 获取上下文（新的锁范围）
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };

        // 3. 获取读锁
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_context_guard = context_read
            .as_any()
            .downcast_ref::<ExchangeEngineContext>()
            .unwrap();

        let exchange = exchange_engine_context_guard.get_exchange_ref(&params.account_id).await.unwrap();

        // 先获取历史k线
        // k线长度设置
        exchange.get_kline_series(&params.symbol, params.interval.clone(), params.cache_size).await?;
        // 再订阅k线流
        exchange.subscribe_kline_stream(&params.symbol, params.interval.clone(), params.frequency).await.unwrap();
        // 获取socket流
        exchange.get_socket_stream().await.unwrap();

        let request_id = params.request_id;
        tracing::debug!("市场数据引擎订阅K线流成功, 请求节点:{}, 请求id: {}", params.node_id, request_id);

        // 都成功后，发送响应事件
        let response_event = ResponseEvent::MarketEngine(MarketEngineResponse::SubscribeKlineStreamSuccess(SubscribeKlineStreamSuccessResponse {
            exchange: params.exchange,
            symbol: params.symbol,
            interval: params.interval,
            response_timestamp: get_utc8_timestamp_millis(),
            response_id: request_id,
        }));
        self.get_event_publisher().publish(response_event.clone().into()).unwrap();
        Ok(())
    }


    async fn unsubscribe_kline_stream(&self, params: UnsubscribeKlineStreamParams) -> Result<(), String> {
        tracing::debug!("市场数据引擎取消订阅K线流: {:?}", params);

        // 1. 先检查注册状态
        let exchange_is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(&params.account_id).await
        };

        if !exchange_is_registered {
            return Err(format!("交易所 {:?} 未注册", params.exchange));
        }
        
        // 2. 获取上下文（新的锁范围）
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };

        // 3. 获取读锁
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_context_guard = context_read
            .as_any()
            .downcast_ref::<ExchangeEngineContext>()
            .unwrap();

        let exchange = exchange_engine_context_guard.get_exchange_ref(&params.account_id).await.unwrap();
        exchange.unsubscribe_kline_stream(&params.symbol, params.interval.clone(), params.frequency).await.unwrap();

        let request_id = params.request_id;
        let response_event = ResponseEvent::MarketEngine(MarketEngineResponse::UnsubscribeKlineStreamSuccess(UnsubscribeKlineStreamSuccessResponse {
            exchange: params.exchange,
            symbol: params.symbol,
            interval: params.interval,
            response_timestamp: get_utc8_timestamp_millis(),
            response_id: request_id,
        }));
        self.get_event_publisher().publish(response_event.clone().into()).unwrap();
        Ok(())
    }

    // pub async fn get_ticker_price(&self, exchange: Exchange, symbol: String) -> Result<serde_json::Value, String> {
    //     match exchange {
    //         Exchange::Binance => {
    //             let state = self.context.read().await;
    //             let binance = state.exchanges.get(&exchange).unwrap();
    //             let ticker_price = binance.get_ticker_price(&symbol).await.unwrap();
    //             Ok(ticker_price)
    //         }

    //         _ => {
    //             return Err("不支持的交易所".to_string());
    //         }
    //     }
    // }



}
