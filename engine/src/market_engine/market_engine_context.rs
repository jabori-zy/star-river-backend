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
use event_center::response::market_engine_response::{MarketEngineResponse, SubscribeKlineStreamResponse, UnsubscribeKlineStreamResponse, GetKlineHistoryResponse};
use event_center::command::market_engine_command::MarketEngineCommand;
use event_center::command::cache_engine_command::AddCacheKeyParams;
use types::cache::{Key, key::KlineKey};
use utils::get_utc8_timestamp_millis;
use types::market::KlineInterval;
use tokio::sync::Mutex;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use crate::market_engine::market_engine_type::KlineSubKey;
use std::collections::HashMap;
use types::custom_type::{StrategyId, AccountId};
use event_center::{EventReceiver, CommandReceiver, CommandPublisher, EventPublisher};
use tokio::sync::oneshot;
use types::strategy::TimeRange;
use types::market::Kline;
use event_center::exchange_event::{ExchangeEvent, ExchangeKlineHistoryUpdateEvent, ExchangeKlineSeriesUpdateEvent};

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
            Command::MarketEngine(MarketEngineCommand::GetKlineHistory(params)) => {
                let kline_history = self.get_kline_history(params.strategy_id, params.account_id, params.exchange.clone(), params.symbol.clone(), params.interval.clone(), params.time_range.clone()).await.unwrap();
                
                // 发布k线历史更新事件
                let exchange_kline_history_update_event = ExchangeEvent::ExchangeKlineHistoryUpdate(ExchangeKlineHistoryUpdateEvent {
                    exchange: params.exchange.clone(),
                    symbol: params.symbol.clone(),
                    interval: params.interval.clone(),
                    time_range: params.time_range.clone(),
                    kline_history: kline_history,
                    event_timestamp: get_utc8_timestamp_millis(),
                });
                self.get_event_publisher().publish(exchange_kline_history_update_event.into()).await.unwrap();
                
                let get_kline_history_response = MarketEngineResponse::GetKlineHistory(GetKlineHistoryResponse {
                    code: 0,
                    message: "success".to_string(),
                    exchange: params.exchange,
                    symbol: params.symbol,
                    interval: params.interval,
                    response_timestamp: get_utc8_timestamp_millis(),
                });
                params.responder.send(get_kline_history_response.into()).unwrap();
            }
            _ => {}
        }
    }


}

impl MarketEngineContext {

    async fn add_kline_key(&self, strategy_id: i32, exchange: Exchange, symbol: String, interval: KlineInterval, start_time: Option<String>, end_time: Option<String>, max_size: u32) {
        // 调用缓存器的订阅事件
        let cache_key = Key::Kline(KlineKey {
            exchange,
            symbol,
            interval,
            start_time,
            end_time,
        });
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = AddCacheKeyParams {
            strategy_id,
            key: cache_key,
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

    async fn add_history_kline_cache_key(&self, strategy_id: i32, exchange: Exchange, symbol: String, interval: KlineInterval, time_range: TimeRange) {
        // 调用缓存器的订阅事件
        let cache_key = Key::Kline(KlineKey {
            exchange: exchange,
            symbol: symbol.to_string(),
            interval: interval.clone(),
            start_time: Some(time_range.start_date.to_string()),
            end_time: Some(time_range.end_date.to_string()),
        });
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = AddCacheKeyParams {
            strategy_id,
            key: cache_key,
            max_size: None,
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

    async fn exchange_is_registered(&self, account_id: AccountId) -> bool {
        let exchange_engine_guard = self.exchange_engine.lock().await;
        exchange_engine_guard.is_registered(&account_id).await
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
        self.add_kline_key(strategy_id, exchange.clone(), symbol.clone(), interval.clone(), None, None, cache_size).await;

        // 1. 先检查注册状态
        let is_registered = self.exchange_is_registered(account_id).await;

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

        let exchange_client = exchange_engine_context_guard.get_exchange_ref(&account_id).await.unwrap();

        // 先获取历史k线
        // 初始的k线
        let initail_kline_series = exchange_client.get_kline_series(&symbol, interval.clone(), cache_size).await.map_err(|e| e.to_string())?;
        let exchange_klineseries_update = ExchangeKlineSeriesUpdateEvent {
            exchange: exchange,
            event_timestamp: get_utc8_timestamp_millis(),
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            kline_series: initail_kline_series.clone(),
        };
        let exchange_klineseries_update_event = ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update);
        self.get_event_publisher().publish(exchange_klineseries_update_event.into()).await.unwrap();
        // 再订阅k线流
        exchange_client.subscribe_kline_stream(&symbol, interval.clone(), frequency).await.unwrap();
        // 获取socket流
        exchange_client.get_socket_stream().await.unwrap();

        
        // self.get_event_publisher().publish(response_event.clone().into()).unwrap();
        Ok(())
    }


    async fn unsubscribe_kline_stream(&self, strategy_id: StrategyId, account_id: AccountId, exchange: Exchange, symbol: String, interval: KlineInterval, frequency: u32) -> Result<(), String> {
        // tracing::debug!("市场数据引擎取消订阅K线流: {:?}", params);

        // 1. 先检查注册状态
        let exchange_is_registered = self.exchange_is_registered(account_id).await;

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

    async fn get_kline_history(&self, strategy_id: StrategyId, account_id: AccountId, exchange: Exchange, symbol: String, interval: KlineInterval, time_range: TimeRange) -> Result<Vec<Kline>, String> {
        // 添加缓存key
        self.add_history_kline_cache_key(strategy_id, exchange.clone(), symbol.clone(), interval.clone(), time_range.clone()).await;
        
        // 1. 先检查注册状态
        let exchange_is_registered = self.exchange_is_registered(account_id).await;

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
        let kline_history = exchange.get_kline_history(&symbol, interval.clone(), time_range).await.unwrap();
        
        Ok(kline_history)
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
