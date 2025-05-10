
use types::indicator::IndicatorConfig;
use types::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeLiveConfig {
    pub indicator_config: IndicatorConfig,
    pub symbol: String,
    pub interval: KlineInterval,
    pub exchange: Exchange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeBacktestConfig {
    pub indicator_config: IndicatorConfig,
    pub symbol: String,
    pub interval: KlineInterval,
    pub exchange: Exchange,
}

#[derive(Debug, Clone)]
pub struct IndicatorNodeSimulateConfig {
    pub indicator_config: IndicatorConfig,
    pub symbol: String,
    pub interval: KlineInterval,
    pub exchange: Exchange,
}