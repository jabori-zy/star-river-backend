mod symbol_handler;
mod event_handler;


use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use star_river_core::custom_type::StrategyId;
use super::subkey::KlineSubKey;
use engine_core::EngineBaseContext;
use super::state_machine::MarketEngineAction;
use exchange_engine::ExchangeEngine;
use crate::state_machine::{MarketEngineStateMachine, market_engine_transition};
use engine_core::state_machine::EngineRunState;
use engine_core::context_trait::EngineContextTrait;
use star_river_core::custom_type::AccountId;
use star_river_core::exchange::Exchange;
use star_river_core::kline::KlineInterval;
use strategy_core::strategy::TimeRange;
use star_river_core::kline::Kline;
use crate::error::MarketEngineError;
use snafu::Report;
use engine_core::EngineContextAccessor;
use star_river_core::engine::EngineName;





#[derive(Debug, Clone)]
pub struct MarketEngineContext {
    pub base_context: EngineBaseContext<MarketEngineAction>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,                         // 交易所引擎
    pub subscribe_klines: Arc<Mutex<HashMap<KlineSubKey, Vec<StrategyId>>>>, // 已订阅的k线
}


impl MarketEngineContext {
    pub fn new(exchange_engine: Arc<Mutex<ExchangeEngine>>) -> Self {

        let state_machine = MarketEngineStateMachine::new(
            EngineName::MarketEngine.to_string(),
            EngineRunState::Created,
            market_engine_transition
        );
        let base_context = EngineBaseContext::new(
            EngineName::MarketEngine,
            state_machine
        );
        
        Self {
            base_context,
            exchange_engine,
            subscribe_klines: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}


impl EngineContextTrait for MarketEngineContext {
    type Action = MarketEngineAction;
    fn base_context(&self) -> &EngineBaseContext<MarketEngineAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineBaseContext<MarketEngineAction> {
        &mut self.base_context
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

    /// 检查交易所是否已注册
    async fn exchange_is_registered(&self, account_id: AccountId) -> bool {
        let exchange_engine_guard = self.exchange_engine.lock().await;
        exchange_engine_guard
            .with_ctx_read(|ctx| ctx.is_registered(&account_id))
            .await
    }

    /// 订阅 K 线流
    // async fn subscribe_kline_stream(
    //     &self,
    //     _strategy_id: StrategyId,
    //     account_id: AccountId,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     cache_size: u32,
    //     frequency: u32,
    // ) -> Result<(), String> {
    //     // 1. 先检查注册状态
    //     let is_registered = self.exchange_is_registered(account_id).await;
    //     if !is_registered {
    //         return Err(format!("交易所 {:?} 未注册", exchange));
    //     }

    //     // 2. 使用 with_ctx_read_async 访问交易所上下文
    //     let exchange_engine_guard = self.exchange_engine.lock().await;

    //     exchange_engine_guard
    //         .with_ctx_read_async(|ctx| {
    //             let symbol = symbol.clone();
    //             let interval_clone = interval.clone();
    //             Box::pin(async move {
    //                 // 获取交易所客户端
    //                 let exchange_client = ctx.get_exchange_instance(&account_id).await
    //                     .map_err(|e| e.to_string())?;

    //                 // 先获取历史 k 线
    //                 let initial_kline_series = exchange_client
    //                     .kline_series(&symbol, interval_clone.clone(), cache_size)
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 // 发布初始 k 线序列更新事件
    //                 let exchange_klineseries_update = ExchangeKlineSeriesUpdateEvent::new(
    //                     exchange,
    //                     symbol.clone(),
    //                     interval_clone.clone().into(),
    //                     initial_kline_series.clone(),
    //                 );
    //                 let exchange_klineseries_update_event =
    //                     ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update);
    //                 EventCenterSingleton::publish(exchange_klineseries_update_event.into())
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 // 订阅 k 线流
    //                 exchange_client
    //                     .subscribe_kline_stream(&symbol, interval_clone.clone(), frequency)
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 // 获取 socket 流
    //                 exchange_client
    //                     .get_socket_stream()
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 Ok::<(), String>(())
    //             })
    //         })
    //         .await?;

    //     Ok(())
    // }

    /// 取消订阅 K 线流
    // async fn unsubscribe_kline_stream(
    //     &self,
    //     _strategy_id: StrategyId,
    //     account_id: AccountId,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     frequency: u32,
    // ) -> Result<(), String> {
    //     // 1. 先检查注册状态
    //     let is_registered = self.exchange_is_registered(account_id).await;
    //     if !is_registered {
    //         return Err(format!("交易所 {:?} 未注册", exchange));
    //     }

    //     // 2. 使用 with_ctx_read_async 访问交易所上下文
    //     let exchange_engine_guard = self.exchange_engine.lock().await;

    //     exchange_engine_guard
    //         .with_ctx_read_async(|ctx| {
    //             let symbol = symbol.clone();
    //             let interval_clone = interval.clone();
    //             Box::pin(async move {
    //                 // 获取交易所客户端
    //                 let exchange_client = ctx.get_exchange_instance(&account_id).await
    //                     .map_err(|e| e.to_string())?;

    //                 // 取消订阅 k 线流
    //                 exchange_client
    //                     .unsubscribe_kline_stream(&symbol, interval_clone, frequency)
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 Ok::<(), String>(())
    //             })
    //         })
    //         .await?;

    //     Ok(())
    // }

    /// 获取历史 K 线数据
    async fn get_kline_history(
        &self,
        account_id: AccountId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, String> {
        // 1. 先检查注册状态
        let is_registered = self.exchange_is_registered(account_id).await;
        if !is_registered {
            return Err(format!("交易所 {:?} 未注册", exchange));
        }

        // 2. 使用 with_ctx_read_async 访问交易所上下文
        let exchange_engine_guard = self.exchange_engine.lock().await;

        let kline_history = exchange_engine_guard
            .with_ctx_read_async(|ctx| {
                let symbol = symbol.clone();
                let interval_clone = interval.clone();
                Box::pin(async move {
                    // 获取交易所客户端
                    let exchange_client = ctx.get_exchange_instance(&account_id).await
                        .map_err(|e| e.to_string())?;

                    // 获取历史 k 线数据
                    let kline_history = exchange_client
                        .kline_history(&symbol, interval_clone, time_range)
                        .await
                        .map_err(|e| {
                            let report = Report::from_error(&e);
                            tracing::error!("{}", report);
                            e.to_string()
                        })?;

                    Ok::<Vec<Kline>, String>(kline_history)
                })
            })
            .await?;

        Ok(kline_history)
    }

    /// 获取支持的 K 线时间间隔
    pub async fn get_support_kline_intervals(&self, account_id: AccountId) -> Result<Vec<KlineInterval>, MarketEngineError> {
        let exchange_engine_guard = self.exchange_engine.lock().await;

        let support_kline_intervals = exchange_engine_guard
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    // 获取交易所客户端
                    let exchange_client = ctx.get_exchange_instance(&account_id).await?;

                    // 获取支持的 k 线时间间隔
                    let support_kline_intervals = exchange_client.support_kline_intervals();

                    Ok::<Vec<KlineInterval>, MarketEngineError>(support_kline_intervals)
                })
            })
            .await?;

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