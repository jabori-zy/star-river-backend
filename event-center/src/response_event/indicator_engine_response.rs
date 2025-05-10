
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::{Exchange, KlineInterval};
use types::indicator::{IndicatorConfig, IndicatorData};
use types::custom_type::{StrategyId, NodeId};
use strum::Display;
use types::indicator::Indicator;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorEngineResponse {
    #[strum(serialize = "register-indicator-response")]
    RegisterIndicatorResponse(RegisterIndicatorResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateIndicatorResponse {
    pub code: i32,
    pub message: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
    pub indicator_series: Vec<Indicator>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterIndicatorResponse {
    pub code: i32,
    pub message: String,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
    pub response_timestamp: i64,
    pub response_id: Uuid,
}
