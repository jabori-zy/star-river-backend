mod symbol_handler;


use crate::EngineName;
use crate::exchange_engine::ExchangeEngine;
use crate::exchange_engine::exchange_engine_context::ExchangeEngineContext;
use crate::market_engine::market_engine_type::KlineSubKey;
use crate::{Engine, EngineContext};
use async_trait::async_trait;
use event_center::EventCenterSingleton;
use event_center::communication::Command;
use event_center::communication::engine::EngineCommand;
use event_center::communication::engine::market_engine::*;
use event_center::event::Event;
use event_center::event::{
    ExchangeEvent,
    exchange_event::ExchangeKlineSeriesUpdateEvent,
};
use snafu::Report;
use star_river_core::custom_type::{AccountId, StrategyId};
use star_river_core::market::Exchange;
use star_river_core::market::Kline;
use star_river_core::market::KlineInterval;
use star_river_core::market::Symbol;
use star_river_core::strategy::TimeRange;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use star_river_core::error::engine_error::market_engine_error::*;


#[derive(Debug)]
pub struct MarketEngineContext {
    pub engine_name: EngineName,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,                         // 交易所引擎
    pub subscribe_klines: Arc<Mutex<HashMap<KlineSubKey, Vec<StrategyId>>>>, // 已订阅的k线
}

impl Clone for MarketEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            exchange_engine: self.exchange_engine.clone(),
            subscribe_klines: self.subscribe_klines.clone(),
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

    async fn handle_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        
        match command {
            EngineCommand::MarketEngine(MarketEngineCommand::SubscribeKlineStream(cmd)) => {
                self.subscribe_kline_stream(
                    cmd.strategy_id,
                    cmd.account_id,
                    cmd.exchange.clone(),
                    cmd.symbol.clone(),
                    cmd.interval.clone(),
                    cmd.cache_size,
                    cmd.frequency,
                )
                .await
                .unwrap();
                tracing::debug!("市场数据引擎订阅K线流成功, 请求节点: {}", cmd.node_id);

                let payload = SubscribeKlineStreamRespPayload::new(cmd.exchange.clone(), cmd.symbol.clone(), cmd.interval.clone());
                let response = SubscribeKlineStreamResponse::success(Some(payload));
                cmd.respond(response);
            }

            EngineCommand::MarketEngine(MarketEngineCommand::UnsubscribeKlineStream(cmd)) => {
                self.unsubscribe_kline_stream(
                    cmd.strategy_id,
                    cmd.account_id,
                    cmd.exchange.clone(),
                    cmd.symbol.clone(),
                    cmd.interval.clone(),
                    cmd.frequency,
                )
                .await
                .unwrap();
                let payload = UnsubscribeKlineStreamRespPayload::new(cmd.exchange.clone(), cmd.symbol.clone(), cmd.interval.clone());
                let response = UnsubscribeKlineStreamResponse::success(Some(payload));
                cmd.respond(response);
            }
            EngineCommand::MarketEngine(MarketEngineCommand::GetKlineHistory(cmd)) => {
                let kline_history = self
                    .get_kline_history(
                        cmd.strategy_id,
                        cmd.account_id,
                        cmd.exchange.clone(),
                        cmd.symbol.clone(),
                        cmd.interval.clone(),
                        cmd.time_range.clone(),
                    )
                    .await
                    .unwrap();

                // 发布k线历史更新事件
                // let exchange_kline_history_update_event = ExchangeKlineHistoryUpdateEvent::new(
                //     params.exchange.clone(),
                //     params.symbol.clone(),
                //     params.interval.clone(),
                //     params.time_range.clone(),
                //     kline_history,
                // );
                // let exchange_kline_history_update_event =
                //     ExchangeEvent::ExchangeKlineHistoryUpdate(exchange_kline_history_update_event);
                // EventCenterSingleton::publish(exchange_kline_history_update_event.into())
                //     .await
                //     .unwrap();
                let payload =
                    GetKlineHistoryRespPayload::new(cmd.exchange.clone(), cmd.symbol.clone(), cmd.interval.clone(), kline_history);
                let resp = GetKlineHistoryResponse::success(Some(payload));
                cmd.respond(resp);
            }
            EngineCommand::MarketEngine(MarketEngineCommand::GetSymbolInfo(cmd)) => {
                let result = self.get_symbol(cmd.account_id, cmd.symbol.clone()).await;
                match result {
                    Ok(symbol) => {
                        let payload = GetSymbolInfoRespPayload::new(symbol);
                        let resp = GetSymbolInfoResponse::success(Some(payload));
                        cmd.respond(resp);
                    }
                    Err(e) => {
                        let resp = GetSymbolInfoResponse::error(Arc::new(e));
                        cmd.respond(resp);
                    }
                }
            }
            _ => {}
        }
    }
}

impl MarketEngineContext {
    // async fn add_kline_key(
    //     &self,
    //     strategy_id: i32,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     start_time: Option<String>,
    //     end_time: Option<String>,
    //     max_size: u32,
    // ) {
    //     // 调用缓存器的订阅事件
    //     let key = KlineKey {
    //         exchange,
    //         symbol,
    //         interval,
    //         start_time,
    //         end_time,
    //     };
    //     let (resp_tx, resp_rx) = oneshot::channel();
    //     let payload = AddKlineKeyCmdPayload::new(strategy_id, key, Some(max_size), Duration::from_millis(10));
    //     let cmd: CacheEngineCommand = AddKlineKeyCommand::new(format!("strategy_{}", strategy_id), resp_tx, Some(payload)).into();

    //     // self.get_command_publisher().send(add_key_command.into()).await.unwrap();
    //     EventCenterSingleton::send_command(cmd.into()).await.unwrap();

    //     let response_event = resp_rx.await.unwrap();
    //     tracing::debug!("市场数据引擎添加缓存key成功, 请求id: {:?}", response_event);

    //     // self.get_event_publisher().publish(command_event.clone().into()).unwrap();
    // }

    // async fn add_history_kline_key(
    //     &self,
    //     strategy_id: i32,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     time_range: TimeRange,
    // ) {
    //     // 调用缓存器的订阅事件
    //     let key = KlineKey {
    //         exchange: exchange,
    //         symbol: symbol.to_string(),
    //         interval: interval.clone(),
    //         start_time: Some(time_range.start_date.to_string()),
    //         end_time: Some(time_range.end_date.to_string()),
    //     };
    //     let (resp_tx, resp_rx) = oneshot::channel();
    //     let payload = AddKlineKeyCmdPayload::new(strategy_id, key, None, Duration::from_millis(10));
    //     let cmd: CacheEngineCommand = AddKlineKeyCommand::new(format!("strategy_{}", strategy_id), resp_tx, Some(payload)).into();

    //     // self.get_command_publisher().send(add_key_command.into()).await.unwrap();
    //     EventCenterSingleton::send_command(cmd.into()).await.unwrap();

    //     let response_event = resp_rx.await.unwrap();
    //     tracing::debug!("市场数据引擎添加缓存key成功, 请求id: {:?}", response_event);

    //     // self.get_event_publisher().publish(command_event.clone().into()).unwrap();
    // }

    async fn exchange_is_registered(&self, account_id: AccountId) -> bool {
        let exchange_engine_guard = self.exchange_engine.lock().await;
        exchange_engine_guard.is_registered(&account_id).await
    }

    async fn subscribe_kline_stream(
        &self,
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
        // self.add_kline_key(
        //     strategy_id,
        //     exchange.clone(),
        //     symbol.clone(),
        //     interval.clone(),
        //     None,
        //     None,
        //     cache_size,
        // )
        // .await;

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
        let exchange_engine_context_guard = context_read.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();

        let exchange_client = exchange_engine_context_guard.get_exchange_ref(&account_id).await.unwrap();

        // 先获取历史k线
        // 初始的k线
        let initail_kline_series = exchange_client
            .get_kline_series(&symbol, interval.clone(), cache_size)
            .await
            .map_err(|e| e.to_string())?;
        let exchange_klineseries_update =
            ExchangeKlineSeriesUpdateEvent::new(exchange, symbol.to_string(), interval.clone().into(), initail_kline_series.clone());
        let exchange_klineseries_update_event = ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update);
        EventCenterSingleton::publish(exchange_klineseries_update_event.into())
            .await
            .unwrap();

        // 再订阅k线流
        exchange_client
            .subscribe_kline_stream(&symbol, interval.clone(), frequency)
            .await
            .unwrap();
        // 获取socket流
        exchange_client.get_socket_stream().await.unwrap();

        Ok(())
    }

    async fn unsubscribe_kline_stream(
        &self,
        strategy_id: StrategyId,
        account_id: AccountId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        frequency: u32,
    ) -> Result<(), String> {
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
        let exchange_engine_context_guard = context_read.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();

        let exchange = exchange_engine_context_guard.get_exchange_ref(&account_id).await.unwrap();
        exchange
            .unsubscribe_kline_stream(&symbol, interval.clone(), frequency)
            .await
            .unwrap();

        Ok(())
    }

    async fn get_kline_history(
        &self,
        strategy_id: i32,
        account_id: AccountId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, String> {
        // 添加缓存key
        // self.add_history_kline_key(
        //     strategy_id,
        //     exchange.clone(),
        //     symbol.clone(),
        //     interval.clone(),
        //     time_range.clone(),
        // )
        // .await;

        // 1. 先检查注册状态
        let exchange_is_registered = self.exchange_is_registered(account_id).await;

        if !exchange_is_registered {
            return Err(format!("exchange {:?} is not registered", exchange));
        }

        // 2. 获取上下文
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };

        // 3. 获取读锁
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_ctx_guard = context_read.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();

        let exchange = exchange_engine_ctx_guard.get_exchange_ref(&account_id).await.unwrap();
        let kline_history = exchange.get_kline_history(&symbol, interval.clone(), time_range).await;
        if let Err(e) = kline_history {
            let report = Report::from_error(&e);
            tracing::error!("{}", report);
            return Err(e.to_string());
        }
        let kline_history = kline_history.unwrap();
        Ok(kline_history)
    }


    pub async fn get_support_kline_intervals(&self, account_id: AccountId) -> Result<Vec<KlineInterval>, MarketEngineError> {
        let exchange_engine_context = {
            let exchange_engine_guard = self.exchange_engine.lock().await;
            exchange_engine_guard.get_context()
        };
        let context_read = exchange_engine_context.read().await;
        let exchange_engine_context_guard = context_read.as_any().downcast_ref::<ExchangeEngineContext>().unwrap();

        let exchange = exchange_engine_context_guard.get_exchange_ref(&account_id).await?;
        let support_kline_intervals = exchange.get_support_kline_intervals();
        Ok(support_kline_intervals)
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
