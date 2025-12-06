use derive_more::From;
use exchange_client::{binance::Binance, metatrader5::MetaTrader5};
use exchange_core::{
    exchange_trait::{Exchange as ExchangeTrait, ExchangeMarketDataExt, ExchangeSymbolExt},
    state_machine::ExchangeRunState,
};
use star_river_core::{
    exchange::Exchange as ExchangeType,
    instrument::Symbol,
    kline::{Kline, KlineInterval},
    system::TimeRange,
};

use crate::error::ExchangeEngineError;

#[derive(Debug, From)]
pub enum Exchange {
    Binance(Binance),
    MetaTrader5(MetaTrader5),
}

impl Exchange {
    pub async fn exchange_type(&self) -> ExchangeType {
        match self {
            Exchange::Binance(exchange) => exchange.exchange_type().await,
            Exchange::MetaTrader5(exchange) => exchange.exchange_type().await,
        }
    }

    pub async fn run_state(&self) -> ExchangeRunState {
        match self {
            Exchange::Binance(exchange) => exchange.run_state().await,
            Exchange::MetaTrader5(exchange) => exchange.run_state().await,
        }
    }

    pub async fn is_in_state(&self, state: &ExchangeRunState) -> bool {
        match self {
            Exchange::Binance(exchange) => exchange.is_in_state(state).await,
            Exchange::MetaTrader5(exchange) => exchange.is_in_state(state).await,
        }
    }

    pub async fn symbol_list(&self) -> Result<Vec<Symbol>, ExchangeEngineError> {
        match self {
            Exchange::Binance(exchange) => Ok(exchange.symbol_list().await?),
            Exchange::MetaTrader5(exchange) => Ok(exchange.symbol_list().await?),
        }
    }

    pub async fn symbol(&self, symbol: String) -> Result<Symbol, ExchangeEngineError> {
        match self {
            Exchange::Binance(exchange) => Ok(exchange.symbol(symbol).await?),
            Exchange::MetaTrader5(exchange) => Ok(exchange.symbol(symbol).await?),
        }
    }

    pub async fn kline_series(&self, symbol: &String, interval: KlineInterval, cache_size: u32) -> Result<Vec<Kline>, ExchangeEngineError> {
        match self {
            Exchange::Binance(exchange) => Ok(exchange.kline_series(symbol, interval, cache_size).await?),
            Exchange::MetaTrader5(exchange) => Ok(exchange.kline_series(symbol, interval, cache_size).await?),
        }
    }

    pub async fn kline_history(
        &self,
        symbol: &String,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, ExchangeEngineError> {
        match self {
            Exchange::Binance(exchange) => Ok(exchange.kline_history(symbol, interval, time_range).await?),
            Exchange::MetaTrader5(exchange) => Ok(exchange.kline_history(symbol, interval, time_range).await?),
        }
    }

    pub fn support_kline_intervals(&self) -> Vec<KlineInterval> {
        match self {
            Exchange::Binance(exchange) => exchange.support_kline_intervals(),
            Exchange::MetaTrader5(exchange) => exchange.support_kline_intervals(),
        }
    }
}
