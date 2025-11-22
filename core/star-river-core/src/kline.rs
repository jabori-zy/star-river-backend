use deepsize::DeepSizeOf;
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::system::DateTimeUtc;

// k线间隔
#[derive(Clone, Serialize, Deserialize, Display, EnumString, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, ToSchema)]
pub enum KlineInterval {
    #[strum(serialize = "1m")]
    #[serde(rename = "1m")]
    Minutes1,
    #[strum(serialize = "2m")]
    #[serde(rename = "2m")]
    Minutes2,
    #[strum(serialize = "3m")]
    #[serde(rename = "3m")]
    Minutes3,
    #[strum(serialize = "4m")]
    #[serde(rename = "4m")]
    Minutes4,
    #[strum(serialize = "5m")]
    #[serde(rename = "5m")]
    Minutes5,
    #[strum(serialize = "6m")]
    #[serde(rename = "6m")]
    Minutes6,
    #[strum(serialize = "10m")]
    #[serde(rename = "10m")]
    Minutes10,
    #[strum(serialize = "12m")]
    #[serde(rename = "12m")]
    Minutes12,
    #[strum(serialize = "15m")]
    #[serde(rename = "15m")]
    Minutes15,
    #[strum(serialize = "20m")]
    #[serde(rename = "20m")]
    Minutes20,
    #[strum(serialize = "30m")]
    #[serde(rename = "30m")]
    Minutes30,
    #[strum(serialize = "1h")]
    #[serde(rename = "1h")]
    Hours1,
    #[strum(serialize = "2h")]
    #[serde(rename = "2h")]
    Hours2,
    #[strum(serialize = "3h")]
    #[serde(rename = "3h")]
    Hours3,
    #[strum(serialize = "4h")]
    #[serde(rename = "4h")]
    Hours4,
    #[strum(serialize = "6h")]
    #[serde(rename = "6h")]
    Hours6,
    #[strum(serialize = "8h")]
    #[serde(rename = "8h")]
    Hours8,
    #[strum(serialize = "12h")]
    #[serde(rename = "12h")]
    Hours12,
    #[strum(serialize = "1d")]
    #[serde(rename = "1d")]
    Days1,
    #[strum(serialize = "1w")]
    #[serde(rename = "1w")]
    Weeks1,
    #[strum(serialize = "1M")]
    #[serde(rename = "1M")]
    Months1,
}

impl KlineInterval {
    pub fn to_seconds(&self) -> i64 {
        match self {
            KlineInterval::Minutes1 => 60,
            KlineInterval::Minutes2 => 120,
            KlineInterval::Minutes3 => 180,
            KlineInterval::Minutes4 => 240,
            KlineInterval::Minutes5 => 300,
            KlineInterval::Minutes6 => 360,
            KlineInterval::Minutes10 => 600,
            KlineInterval::Minutes12 => 720,
            KlineInterval::Minutes15 => 900,
            KlineInterval::Minutes20 => 1200,
            KlineInterval::Minutes30 => 1800,
            KlineInterval::Hours1 => 3600,
            KlineInterval::Hours2 => 7200,
            KlineInterval::Hours3 => 10800,
            KlineInterval::Hours4 => 14400,
            KlineInterval::Hours6 => 21600,
            KlineInterval::Hours8 => 28800,
            KlineInterval::Hours12 => 43200,
            KlineInterval::Days1 => 86400,
            KlineInterval::Weeks1 => 604800,
            KlineInterval::Months1 => 2629746,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, DeepSizeOf)]
pub struct Kline {
    pub datetime: DateTimeUtc, // 时间戳，单位为毫秒
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Kline {
    pub fn new(datetime: DateTimeUtc, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            datetime,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn datetime(&self) -> DateTimeUtc {
        self.datetime
    }

    pub fn open(&self) -> f64 {
        self.open
    }

    pub fn high(&self) -> f64 {
        self.high
    }

    pub fn low(&self) -> f64 {
        self.low
    }

    pub fn close(&self) -> f64 {
        self.close
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }
}

impl Kline {
    pub fn get_datetime(&self) -> DateTimeUtc {
        self.datetime
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    pub fn to_list(&self) -> Vec<f64> {
        vec![
            self.datetime.timestamp_millis() as f64,
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
        ]
    }
    pub fn to_json_with_time(&self) -> serde_json::Value {
        json!(
            {
                "timestamp": self.datetime.to_string(),
                "open": self.open,
                "high": self.high,
                "low": self.low,
                "close": self.close,
                "volume": self.volume
            }
        )
    }

    pub fn get_value(&self, key: &str) -> Option<f64> {
        match key {
            "datetime" => Some(self.datetime().timestamp_millis() as f64),
            "open" => Some(self.open()),
            "high" => Some(self.high()),
            "low" => Some(self.low()),
            "close" => Some(self.close()),
            "volume" => Some(self.volume()),
            _ => None,
        }
    }
}
