use super::Key;
use super::KeyTrait;
use crate::error::star_river_error::*;
use crate::exchange::Exchange;
use crate::kline::KlineInterval;
use crate::strategy::TimeRange;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::str::FromStr;
use ta_lib::IndicatorConfig;



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
    fn from(indicator_key: IndicatorKey) -> Self {
        Key::Indicator(indicator_key)
    }
}

impl FromStr for IndicatorKey {
    type Err = StarRiverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 7 {
            return Err(InvalidKeyFormatSnafu { key_str: s.to_string() }.build());
        }

        let exchange = parts[1].parse::<Exchange>()?;
        let symbol = parts[2].to_string();
        let interval = parts[3].parse::<KlineInterval>().context(ParseKlineIntervalFailedSnafu {
            interval: parts[3].to_string(),
        })?;
        let indicator_config = IndicatorConfig::from_str(parts[4]).unwrap();
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

    pub fn get_kline_key(&self) -> KlineKey {
        KlineKey::new(
            self.exchange.clone(),
            self.symbol.clone(),
            self.interval.clone(),
            self.start_time.clone(),
            self.end_time.clone(),
        )
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
    fn get_time_range(&self) -> Option<TimeRange> {
        match (&self.start_time, &self.end_time) {
            (Some(start_time), Some(end_time)) => Some(TimeRange::new(start_time.clone(), end_time.clone())),
            _ => None,
        }
    }
}