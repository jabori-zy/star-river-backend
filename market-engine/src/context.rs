mod event_handler;
mod symbol_handler;

use std::{collections::HashMap, sync::Arc};

use engine_core::{EngineContextAccessor, EngineMetadata, context_trait::EngineContextTrait, state_machine::EngineRunState};
use exchange_engine::{ExchangeEngine, error::ExchangeEngineError};
use star_river_core::{
    custom_type::{AccountId, StrategyId},
    engine::EngineName,
    exchange::Exchange,
    kline::{Kline, KlineInterval},
    system::TimeRange,
};
use tokio::sync::Mutex;

use super::{state_machine::MarketEngineAction, subkey::KlineSubKey};
use crate::{
    error::{ExchangeNotRegisteredSnafu, MarketEngineError},
    state_machine::{MarketEngineStateMachine, market_engine_transition},
};

#[derive(Debug, Clone)]
pub struct MarketEngineContext {
    pub base_context: EngineMetadata<MarketEngineAction>,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,                         // Exchange engine
    pub subscribe_klines: Arc<Mutex<HashMap<KlineSubKey, Vec<StrategyId>>>>, // Subscribed klines
}

impl MarketEngineContext {
    pub fn new(exchange_engine: Arc<Mutex<ExchangeEngine>>) -> Self {
        let state_machine = MarketEngineStateMachine::new(
            EngineName::MarketEngine.to_string(),
            EngineRunState::Created,
            market_engine_transition,
        );
        let base_context = EngineMetadata::new(EngineName::MarketEngine, state_machine);

        Self {
            base_context,
            exchange_engine,
            subscribe_klines: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl EngineContextTrait for MarketEngineContext {
    type Action = MarketEngineAction;
    fn base_context(&self) -> &EngineMetadata<MarketEngineAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineMetadata<MarketEngineAction> {
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
    //     // Call cache subscription event
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
    //     tracing::debug!("Market engine added cache key successfully, request id: {:?}", response_event);

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
    //     // Call cache subscription event
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
    //     tracing::debug!("Market engine added cache key successfully, request id: {:?}", response_event);

    //     // self.get_event_publisher().publish(command_event.clone().into()).unwrap();
    // }

    /// Check if exchange is registered
    async fn exchange_is_registered(&self, account_id: AccountId) -> bool {
        let exchange_engine_guard = self.exchange_engine.lock().await;
        exchange_engine_guard.with_ctx_read(|ctx| ctx.is_registered(&account_id)).await
    }

    /// Subscribe to kline stream
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
    //     // 1. Check registration status first
    //     let is_registered = self.exchange_is_registered(account_id).await;
    //     if !is_registered {
    //         return Err(format!("Exchange {:?} not registered", exchange));
    //     }

    //     // 2. Access exchange context using with_ctx_read_async
    //     let exchange_engine_guard = self.exchange_engine.lock().await;

    //     exchange_engine_guard
    //         .with_ctx_read_async(|ctx| {
    //             let symbol = symbol.clone();
    //             let interval_clone = interval.clone();
    //             Box::pin(async move {
    //                 // Get exchange client
    //                 let exchange_client = ctx.get_exchange_instance(&account_id).await
    //                     .map_err(|e| e.to_string())?;

    //                 // Get historical klines first
    //                 let initial_kline_series = exchange_client
    //                     .kline_series(&symbol, interval_clone.clone(), cache_size)
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 // Publish initial kline series update event
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

    //                 // Subscribe to kline stream
    //                 exchange_client
    //                     .subscribe_kline_stream(&symbol, interval_clone.clone(), frequency)
    //                     .await
    //                     .map_err(|e| e.to_string())?;

    //                 // Get socket stream
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

    /// Unsubscribe from kline stream
    // async fn unsubscribe_kline_stream(
    //     &self,
    //     _strategy_id: StrategyId,
    //     account_id: AccountId,
    //     exchange: Exchange,
    //     symbol: String,
    //     interval: KlineInterval,
    //     frequency: u32,
    // ) -> Result<(), String> {
    //     // 1. Check registration status first
    //     let is_registered = self.exchange_is_registered(account_id).await;
    //     if !is_registered {
    //         return Err(format!("Exchange {:?} not registered", exchange));
    //     }

    //     // 2. Access exchange context using with_ctx_read_async
    //     let exchange_engine_guard = self.exchange_engine.lock().await;

    //     exchange_engine_guard
    //         .with_ctx_read_async(|ctx| {
    //             let symbol = symbol.clone();
    //             let interval_clone = interval.clone();
    //             Box::pin(async move {
    //                 // Get exchange client
    //                 let exchange_client = ctx.get_exchange_instance(&account_id).await
    //                     .map_err(|e| e.to_string())?;

    //                 // Unsubscribe from kline stream
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

    /// Get historical kline data
    async fn get_kline_history(
        &self,
        account_id: AccountId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, MarketEngineError> {
        // 1. Check registration status first
        let is_registered = self.exchange_is_registered(account_id).await;
        if !is_registered {
            return Err(ExchangeNotRegisteredSnafu { account_id, exchange }.build());
        }

        // 2. Access exchange context using with_ctx_read_async
        let exchange_engine_guard = self.exchange_engine.lock().await;

        let kline_history = exchange_engine_guard
            .with_ctx_read_async(|ctx| {
                let symbol = symbol.clone();
                let interval_clone = interval.clone();
                Box::pin(async move {
                    // Get exchange client
                    let exchange_client = ctx.get_exchange_instance(&account_id).await?;

                    // Get historical kline data
                    let kline_history = exchange_client.kline_history(&symbol, interval_clone, time_range).await?;

                    Ok::<Vec<Kline>, ExchangeEngineError>(kline_history)
                })
            })
            .await?;

        Ok(kline_history)
    }

    /// Get supported kline intervals
    pub async fn get_support_kline_intervals(&self, account_id: AccountId) -> Result<Vec<KlineInterval>, MarketEngineError> {
        let exchange_engine_guard = self.exchange_engine.lock().await;

        let support_kline_intervals = exchange_engine_guard
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    // Get exchange client
                    let exchange_client = ctx.get_exchange_instance(&account_id).await?;

                    // Get supported kline intervals
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
    //             return Err("Unsupported exchange".to_string());
    //         }
    //     }
    // }
}
