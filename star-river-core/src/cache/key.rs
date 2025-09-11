use super::Key;
use super::KeyTrait;
use crate::indicator::IndicatorConfig;
use crate::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub struct KlineKey {
//     pub exchange: Exchange,
//     pub symbol: String,
//     pub interval: KlineInterval,
// }
//
// impl From<KlineKey> for Key {
//     fn from(kline_cache_key: KlineKey) -> Self {
//         Key::Kline(kline_cache_key)
//     }
// }
//
// impl FromStr for KlineKey {
//     type Err = String;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let parts: Vec<&str> = s.split('|').collect();
//         if parts.len() != 4 {
//             return Err("Invalid cache key format".to_string());
//         }
//
//         let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
//         let symbol = parts[2].to_string();
//         let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
//         Ok(KlineKey::new(exchange, symbol, interval))
//     }
// }
//
//
//
// impl KlineKey {
//     pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
//         Self { exchange, symbol, interval }
//     }
// }
//
// impl KeyTrait for KlineKey {
//     fn get_key_str(&self) -> String {
//         format!("kline|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string())
//     }
//     fn get_exchange(&self) -> Exchange {
//         self.exchange.clone()
//     }
//     fn get_symbol(&self) -> String {
//         self.symbol.clone()
//     }
//     fn get_interval(&self) -> KlineInterval {
//         self.interval.clone()
//     }
// }
//
//
// #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
// pub struct IndicatorKey {
//     pub exchange: Exchange,
//     pub symbol: String,
//     pub interval: KlineInterval,
//     pub indicator_config: IndicatorConfig,
// }
//
//
// impl FromStr for IndicatorKey {
//     type Err = String;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let parts: Vec<&str> = s.split('|').collect();
//         if parts.len() != 5 {
//             return Err("Invalid cache key format".to_string());
//         }
//
//         let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
//         let symbol = parts[2].to_string();
//         let interval = parts[3].parse::<KlineInterval>().map_err(|e| e.to_string())?;
//         let indicator_config = IndicatorConfig::from_str(parts[4])?;
//         Ok(IndicatorKey::new(exchange, symbol, interval, indicator_config))
//     }
// }
//
//
//
// impl From<IndicatorKey> for Key {
//     fn from(indicator_cache_key: IndicatorKey) -> Self {
//         Key::Indicator(indicator_cache_key)
//     }
// }
//
// impl IndicatorKey {
//     pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, indicator_config: IndicatorConfig) -> Self {
//         Self { exchange, symbol, interval, indicator_config }
//     }
//
//     pub fn get_indicator(&self) -> IndicatorConfig {
//         self.indicator_config.clone()
//     }
// }
//
// impl KeyTrait for IndicatorKey {
//     fn get_key_str(&self) -> String {
//         format!("indicator|{}|{}|{}|{}", self.exchange.to_string(), self.symbol, self.interval.to_string(), self.indicator_config.to_string())
//     }
//     fn get_exchange(&self) -> Exchange {
//         self.exchange.clone()
//     }
//     fn get_symbol(&self) -> String {
//         self.symbol.clone()
//     }
//     fn get_interval(&self) -> KlineInterval {
//         self.interval.clone()
//     }
// }

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KlineKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl From<KlineKey> for Key {
    fn from(history_kline_cache_key: KlineKey) -> Self {
        Key::Kline(history_kline_cache_key)
    }
}

impl FromStr for KlineKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 6 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3]
            .parse::<KlineInterval>()
            .map_err(|e| e.to_string())?;
        // 使用Box::leak将字符串转换为静态引用
        let start_time = Some(parts[4].to_string());
        let end_time = Some(parts[5].to_string());
        Ok(KlineKey::new(
            exchange, symbol, interval, start_time, end_time,
        ))
    }
}

impl KlineKey {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        start_time: Option<String>,
        end_time: Option<String>,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            start_time,
            end_time,
        }
    }
}

impl KeyTrait for KlineKey {
    fn get_key_str(&self) -> String {
        match (&self.start_time, &self.end_time) {
            (Some(start_time), Some(end_time)) => {
                format!(
                    "kline|{}|{}|{}|{}|{}",
                    self.exchange.to_string(),
                    self.symbol,
                    self.interval.to_string(),
                    start_time.clone(),
                    end_time.clone()
                )
            }
            _ => {
                format!(
                    "kline|{}|{}|{}",
                    self.exchange.to_string(),
                    self.symbol,
                    self.interval.to_string()
                )
            }
        }
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IndicatorKey {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub indicator_config: IndicatorConfig,
}

impl From<IndicatorKey> for Key {
    fn from(backtest_indicator_cache_key: IndicatorKey) -> Self {
        Key::Indicator(backtest_indicator_cache_key)
    }
}

impl FromStr for IndicatorKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 7 {
            return Err("Invalid cache key format".to_string());
        }

        let exchange = parts[1].parse::<Exchange>().map_err(|e| e.to_string())?;
        let symbol = parts[2].to_string();
        let interval = parts[3]
            .parse::<KlineInterval>()
            .map_err(|e| e.to_string())?;
        let indicator_config = IndicatorConfig::from_str(parts[4])?;
        let start_time = Some(parts[5].to_string());
        let end_time = Some(parts[6].to_string());
        let kline_key = KlineKey::new(exchange, symbol, interval, start_time, end_time);
        Ok(IndicatorKey::new(kline_key, indicator_config))
    }
}

impl IndicatorKey {
    pub fn new(kline_key: KlineKey, indicator_config: IndicatorConfig) -> Self {
        Self {
            exchange: kline_key.exchange,
            symbol: kline_key.symbol,
            interval: kline_key.interval,
            start_time: kline_key.start_time,
            end_time: kline_key.end_time,
            indicator_config,
        }
    }

    pub fn get_indicator_config(&self) -> IndicatorConfig {
        self.indicator_config.clone()
    }
}

impl KeyTrait for IndicatorKey {
    fn get_key_str(&self) -> String {
        match (&self.start_time, &self.end_time) {
            (Some(start_time), Some(end_time)) => {
                format!(
                    "indicator|{}|{}|{}|{}|{}|{}",
                    self.exchange.to_string(),
                    self.symbol,
                    self.interval.to_string(),
                    self.indicator_config.to_string(),
                    start_time,
                    end_time
                )
            }
            _ => {
                format!(
                    "indicator|{}|{}|{}|{}",
                    self.exchange.to_string(),
                    self.symbol,
                    self.interval.to_string(),
                    self.indicator_config.to_string()
                )
            }
        }
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
