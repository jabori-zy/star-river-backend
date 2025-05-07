
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use types::market::{Exchange, KlineInterval};
use types::indicator::{IndicatorConfig, IndicatorData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndicatorEngineResponse {
    // 计算指标完成
    CalculateIndicatorFinish(CalculateIndicatorResponse),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateIndicatorResponse {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
    pub value: Box<dyn IndicatorData>,
    pub response_timestamp: i64,
    pub response_id: Uuid,
    pub batch_id: String,
}