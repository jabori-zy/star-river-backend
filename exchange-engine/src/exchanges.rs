use exchange_client_new::binance::Binance;
use exchange_client_new::metatrader5::MetaTrader5;

use star_river_core::exchange::Exchange as ExchangeType;
use exchange_core::state_machine::ExchangeRunState;
use exchange_core::exchange_trait::Exchange as ExchangeTrait;
use star_river_core::instrument::Symbol;
use exchange_core::exchange_trait::{ExchangeSymbolExt, ExchangeMarketDataExt};
use star_river_core::kline::{Kline, KlineInterval};
use strategy_core::strategy::TimeRange;
use exchange_client_new::exchange_error::ExchangeError;
use crate::error::ExchangeEngineError;


#[derive(Debug)]
pub enum Exchange {
    Binance(Binance),
    MetaTrader5(MetaTrader5),
}


impl From<Binance> for Exchange {
    fn from(binance: Binance) -> Self {
        Exchange::Binance(binance)
    }
}

impl From<MetaTrader5> for Exchange {
    fn from(metatrader5: MetaTrader5) -> Self {
        Exchange::MetaTrader5(metatrader5)
    }
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
            Exchange::Binance(exchange) => Ok(exchange.symbol(symbol).await.map_err(|e| ExchangeEngineError::from(e))?),
            Exchange::MetaTrader5(exchange) => Ok(exchange.symbol(symbol).await?),
        }
    }


    pub async fn kline_series(&self, symbol: &String, interval: KlineInterval, cache_size: u32) -> Result<Vec<Kline>, ExchangeEngineError> {
        match self {
            Exchange::Binance(exchange) => Ok(exchange.kline_series(symbol, interval, cache_size).await?),
            Exchange::MetaTrader5(exchange) => Ok(exchange.kline_series(symbol, interval, cache_size).await?),
        }
    }


    pub async fn kline_history(&self, symbol: &String, interval: KlineInterval, time_range: TimeRange) -> Result<Vec<Kline>, ExchangeEngineError> {
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