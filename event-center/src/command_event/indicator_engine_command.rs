use serde::{Deserialize, Serialize};
use strum::Display;
use std::fmt::Debug;
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;
use types::indicator::IndicatorConfig;
use types::new_cache::CacheValue;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorEngineCommand {
    #[strum(serialize = "calculate-indicator")]
    CalculateIndicator(CalculateIndicatorParams), // 计算指标
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateIndicatorParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
    pub kline_series: Vec<CacheValue>,
    pub sender: String,
    pub command_timestamp:i64,
    pub request_id: Uuid,
}