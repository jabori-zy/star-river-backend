use crate::market::{Exchange, KlineInterval};
use crate::indicator::IndicatorConfig;
use serde::{Deserialize, Serialize};
use super::CacheKeyTrait;
use super::CacheKey;
use std::str::FromStr;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlineCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl From<KlineCacheKey> for CacheKey {
    fn from(kline_cache_key: KlineCacheKey) -> Self {
        CacheKey::Kline(kline_cache_key)
    }
}

impl FromStr for KlineCacheKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 4 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
        Ok(KlineCacheKey::new(exchange, symbol, interval))
    }
}



impl KlineCacheKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self { exchange, symbol, interval }
    }
}

impl CacheKeyTrait for KlineCacheKey {
    fn get_key(&self) -> String {
        format!("kline|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string())
    }
    fn get_exchange(&self) -> Exchange {
        self.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.interval.clone()
    }
}


#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndicatorCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
}


impl FromStr for IndicatorCacheKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 5 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
        let indicator_config = IndicatorConfig::from_str(parts[4])?;
        Ok(IndicatorCacheKey::new(exchange, symbol, interval, indicator_config))
    }
}



impl From<IndicatorCacheKey> for CacheKey {
    fn from(indicator_cache_key: IndicatorCacheKey) -> Self {
        CacheKey::Indicator(indicator_cache_key)
    }
}

impl IndicatorCacheKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, indicator_config: IndicatorConfig) -> Self {
        Self { exchange, symbol, interval, indicator_config }
    }

    pub fn get_indicator(&self) -> IndicatorConfig {
        self.indicator_config.clone()
    }
}

impl CacheKeyTrait for IndicatorCacheKey {
    fn get_key(&self) -> String {
        format!("indicator|{}|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string(), self.indicator_config.to_string())
    }
    fn get_exchange(&self) -> Exchange {
        self.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.interval.clone()
    }
}



#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BacktestKlineCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: String,
    pub end_time: String,
}


impl From<BacktestKlineCacheKey> for CacheKey {
    fn from(history_kline_cache_key: BacktestKlineCacheKey) -> Self {
        CacheKey::BacktestKline(history_kline_cache_key)
    }
}


impl FromStr for BacktestKlineCacheKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 6 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
        let start_time = parts[4].to_string();
        let end_time = parts[5].to_string();
        Ok(BacktestKlineCacheKey::new(exchange, symbol, interval, start_time, end_time))
    }
}


impl BacktestKlineCacheKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, start_time: String, end_time: String) -> Self {
        Self { exchange, symbol, interval, start_time, end_time }
    }
}

impl CacheKeyTrait for BacktestKlineCacheKey {
    fn get_key(&self) -> String {
        format!("backtest_kline|{}|{}|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string(), self.start_time, self.end_time)
    }
    fn get_exchange(&self) -> Exchange {
        self.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.interval.clone()
    }
}



#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BacktestIndicatorCacheKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
    pub start_time: String,
    pub end_time: String,
}


impl From<BacktestIndicatorCacheKey> for CacheKey {
    fn from(backtest_indicator_cache_key: BacktestIndicatorCacheKey) -> Self {
        CacheKey::BacktestIndicator(backtest_indicator_cache_key)
    }
}



impl FromStr for BacktestIndicatorCacheKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 7 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
        let indicator_config = IndicatorConfig::from_str(parts[4])?;
        let start_time = parts[5].to_string();
        let end_time = parts[6].to_string();
        let kline_cache_key = CacheKey::BacktestKline(BacktestKlineCacheKey::new(exchange, symbol, interval, start_time, end_time));
        Ok(BacktestIndicatorCacheKey::new(kline_cache_key, indicator_config))
    }
}


impl BacktestIndicatorCacheKey {
    pub fn new(kline_cache_key: CacheKey, indicator_config: IndicatorConfig) -> Self {
        match kline_cache_key {
            CacheKey::BacktestKline(backtest_kline_cache_key) => Self { 
                exchange: backtest_kline_cache_key.exchange, 
                symbol: backtest_kline_cache_key.symbol, 
                interval: backtest_kline_cache_key.interval, 
                indicator_config,
                start_time: backtest_kline_cache_key.start_time, 
                end_time: backtest_kline_cache_key.end_time
            },
            _ => panic!("Invalid cache key"),
        }
    }

    pub fn get_indicator_config(&self) -> IndicatorConfig {
        self.indicator_config.clone()
    }
}


impl CacheKeyTrait for BacktestIndicatorCacheKey {
    fn get_key(&self) -> String {
        format!("backtest_indicator|{}|{}|{}|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string(), self.indicator_config.to_string(), self.start_time, self.end_time)
    }
    fn get_exchange(&self) -> Exchange {
        self.exchange.clone()
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_interval(&self) -> KlineInterval {
        self.interval.clone()
    }
}
