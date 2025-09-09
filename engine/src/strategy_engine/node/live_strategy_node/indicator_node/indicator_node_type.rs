use serde::{Deserialize, Serialize};
use types::indicator::IndicatorConfig;
use types::market::{Exchange, KlineInterval};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeLiveConfig {
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
