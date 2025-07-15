use crate::market::{Exchange, KlineInterval};
use crate::indicator::IndicatorConfig;
use serde::{Deserialize, Serialize};
use super::KeyTrait;
use super::Key;
use std::str::FromStr;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct KlineKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl From<KlineKey> for Key {
    fn from(kline_cache_key: KlineKey) -> Self {
        Key::Kline(kline_cache_key)
    }
}

impl FromStr for KlineKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 4 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
        Ok(KlineKey::new(exchange, symbol, interval))
    }
}



impl KlineKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self { exchange, symbol, interval }
    }
}

impl KeyTrait for KlineKey {
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
pub struct IndicatorKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
}


impl FromStr for IndicatorKey {
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
        Ok(IndicatorKey::new(exchange, symbol, interval, indicator_config))
    }
}



impl From<IndicatorKey> for Key {
    fn from(indicator_cache_key: IndicatorKey) -> Self {
        Key::Indicator(indicator_cache_key)
    }
}

impl IndicatorKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, indicator_config: IndicatorConfig) -> Self {
        Self { exchange, symbol, interval, indicator_config }
    }

    pub fn get_indicator(&self) -> IndicatorConfig {
        self.indicator_config.clone()
    }
}

impl KeyTrait for IndicatorKey {
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
pub struct BacktestKlineKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: String,
    pub end_time: String,
}


impl From<BacktestKlineKey> for Key {
    fn from(history_kline_cache_key: BacktestKlineKey) -> Self {
        Key::BacktestKline(history_kline_cache_key)
    }
}


impl FromStr for BacktestKlineKey {
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
        Ok(BacktestKlineKey::new(exchange, symbol, interval, start_time, end_time))
    }
}


impl BacktestKlineKey {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, start_time: String, end_time: String) -> Self {
        Self { exchange, symbol, interval, start_time, end_time }
    }
}

impl KeyTrait for BacktestKlineKey {
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
pub struct BacktestIndicatorKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
    pub start_time: String,
    pub end_time: String,
}


impl From<BacktestIndicatorKey> for Key {
    fn from(backtest_indicator_cache_key: BacktestIndicatorKey) -> Self {
        Key::BacktestIndicator(backtest_indicator_cache_key)
    }
}



impl FromStr for BacktestIndicatorKey {
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
        let kline_cache_key = Key::BacktestKline(BacktestKlineKey::new(exchange, symbol, interval, start_time, end_time));
        Ok(BacktestIndicatorKey::new(kline_cache_key, indicator_config))
    }
}


impl BacktestIndicatorKey {
    pub fn new(kline_cache_key: Key, indicator_config: IndicatorConfig) -> Self {
        match kline_cache_key {
            Key::BacktestKline(backtest_kline_cache_key) => Self { 
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


impl KeyTrait for BacktestIndicatorKey {
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
