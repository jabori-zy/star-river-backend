use tokio::sync::broadcast;
use types::market::Exchange;
use event_center::Event;
use crate::exchange_engine::ExchangeEngine;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use std::any::Any;
use tokio::time::Duration;
use crate::EngineName;
use std::sync::Arc;
use event_center::command::Command;
use event_center::command::cache_engine_command::CacheEngineCommand;
use event_center::response::market_engine_response::{MarketEngineResponse, SubscribeKlineStreamResponse, UnsubscribeKlineStreamResponse};
use event_center::command::market_engine_command::MarketEngineCommand;
use event_center::command::cache_engine_command::AddCacheKeyParams;
use types::cache::{CacheKey, cache_key::KlineCacheKey};
use utils::get_utc8_timestamp_millis;
use types::market::KlineInterval;
use tokio::sync::Mutex;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use crate::market_engine::market_engine_type::KlineSubKey;
use std::collections::HashMap;
use types::custom_type::{StrategyId, AccountId};
use event_center::{EventReceiver, CommandReceiver, CommandPublisher, EventPublisher};
use tokio::sync::oneshot;



#[derive(Debug)]
pub struct MarketEngineContext {
    pub engine_name: EngineName,
    pub event_publisher: EventPublisher, // 事件发布器
    pub event_receiver: Vec<EventReceiver>, // 事件接收器
    pub command_publisher: CommandPublisher, // 命令发布器
    pub command_receiver: Arc<Mutex<CommandReceiver>>, // 命令接收器
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>, // 交易所引擎
    pub subscribe_klines: Arc<Mutex<HashMap<KlineSubKey, Vec<StrategyId>>>>, // 已订阅的k线
}

impl Clone for MarketEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self.event_receiver.iter().map(|receiver| receiver.resubscribe()).collect(),
            exchange_engine: self.exchange_engine.clone(),
            subscribe_klines: self.subscribe_klines.clone(),
            command_publisher: self.command_publisher.clone(),
            command_receiver: self.command_receiver.clone(),
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

    fn get_command_publisher(&self) -> &CommandPublisher {
        &self.command_publisher
    }

    fn get_command_receiver(&self) -> Arc<Mutex<CommandReceiver>> {
        self.command_receiver.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        let _event = event;

    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::MarketEngine(MarketEngineCommand::SubscribeKlineStream(command_params)) => {
                self.subscribe_kline_stream(
                    command_params.strategy_id, 
                    command_params.account_id, 
                    command_params.exchange.clone(), 
                    command_params.symbol.clone(), 
                    command_params.interval.clone(),
                    command_params.cache_size, 
                    command_params.frequency).await.unwrap();
                tracing::debug!("市场数据引擎订阅K线流成功, 请求节点: {}", command_params.node_id);

                // 都成功后，发送响应事件
                let subscribe_kline_stream_response = MarketEngineResponse::SubscribeKlineStream(SubscribeKlineStreamResponse {
                    code: 0,
                    message: "success".to_string(),
                    exchange: command_params.exchange,
                    symbol: command_params.symbol,
                    interval: command_params.interval,
                    response_timestamp: get_utc8_timestamp_millis(),
                });
                command_params.responder.send(subscribe_kline_stream_response.into()).unwrap();
            }

            Command::MarketEngine(MarketEngineCommand::UnsubscribeKlineStream(command_params)) => {
                self.unsubscribe_kline_stream(
                    command_params.strategy_id, 
                    command_params.account_id, 
                    command_params.exchange.clone(), 
                    command_params.symbol.clone(), 
                    command_params.interval.clone(), 
                    command_params.frequency).await.unwrap();
                let unsubscribe_kline_stream_response = MarketEngineResponse::UnsubscribeKlineStream(UnsubscribeKlineStreamResponse {
                    code: 0,
                    message: "success".to_string(),
                    exchange: command_params.exchange,
                    symbol: command_params.symbol,
                    interval: command_params.interval,
                    response_timestamp: get_utc8_timestamp_millis(),
                });
                command_params.responder.send(unsubscribe_kline_stream_response.into()).unwrap();
            }
            _ => {}
        }
    }


}

impl MarketEngineContext {

    async fn add_cache_key(&self, strategy_id: i32, exchange: Exchange, symbol: String, interval: KlineInterval, max_size: u32) {
        // 调用缓存器的订阅事件
        let cache_key = CacheKey::Kline(KlineCacheKey {
            exchange: exchange,
            symbol: symbol.to_string(),
            interval: interval.clone(),
        });
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = AddCacheKeyParams {
            strategy_id,
            cache_key,
            max_size: Some(max_size),
            duration: Duration::from_millis(10),
            sender: format!("strategy_{}", strategy_id),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let add_cache_key_command = CacheEngineCommand::AddCacheKey(params);

        self.get_command_publisher().send(add_cache_key_command.into()).await.unwrap();

        let response_event = resp_rx.await.unwrap();
        tracing::debug!("市场数据引擎添加缓存key成功, 请求id: {:?}", response_event);

        // self.get_event_publisher().publish(command_event.clone().into()).unwrap();
    }


    async fn subscribe_kline_stream(&self, 
        strategy_id: StrategyId,
        account_id: AccountId,
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        cache_size: u32,
        frequency: u32,
    ) -> Result<(), String> {
        // tracing::debug!("市场数据引擎订阅K线流: {:?}", params);
        // 添加缓存key
        self.add_cache_key(strategy_id, exchange.clone(), symbol.clone(), interval.clone(), cache_size).await;

        // 1. 先检查注册状态
        let is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(&account_id).await
        };

        if !is_registered {
            return Err(format!("交易所 {:?} 未注册", exchange));
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

        let exchange = exchange_engine_context_guard.get_exchange_ref(&account_id).await.unwrap();

        // 先获取历史k线
        // k线长度设置
        exchange.get_kline_series(&symbol, interval.clone(), cache_size).await?;
        // 再订阅k线流
        exchange.subscribe_kline_stream(&symbol, interval.clone(), frequency).await.unwrap();
        // 获取socket流
        exchange.get_socket_stream().await.unwrap();

        
        // self.get_event_publisher().publish(response_event.clone().into()).unwrap();
        Ok(())
    }


    async fn unsubscribe_kline_stream(&self, strategy_id: StrategyId, account_id: AccountId, exchange: Exchange, symbol: String, interval: KlineInterval, frequency: u32) -> Result<(), String> {
        // tracing::debug!("市场数据引擎取消订阅K线流: {:?}", params);

        // 1. 先检查注册状态
        let exchange_is_registered = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.is_registered(&account_id).await
        };

        if !exchange_is_registered {
            return Err(format!("交易所 {:?} 未注册", exchange));
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

        let exchange = exchange_engine_context_guard.get_exchange_ref(&account_id).await.unwrap();
        exchange.unsubscribe_kline_stream(&symbol, interval.clone(), frequency).await.unwrap();

        
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
