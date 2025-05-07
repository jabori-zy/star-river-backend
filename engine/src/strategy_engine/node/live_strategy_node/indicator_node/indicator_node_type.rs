
use types::indicator::IndicatorConfig;
use types::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeLiveConfig {
    pub indicator: IndicatorConfig,
    pub symbol: String,
    pub interval: KlineInterval,
    pub exchange: Exchange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeBacktestConfig {
    pub indicator: IndicatorConfig,
    pub symbol: String,
    pub interval: KlineInterval,
    pub exchange: Exchange,
}

#[derive(Debug, Clone)]
pub struct IndicatorNodeSimulateConfig {
    pub indicator: IndicatorConfig,
    pub symbol: String,
    pub interval: KlineInterval,
    pub exchange: Exchange,
}