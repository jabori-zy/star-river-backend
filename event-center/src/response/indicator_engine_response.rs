
use uuid::Uuid;
use types::market::{Exchange, KlineInterval};
use types::indicator::IndicatorConfig;
use types::custom_type::{StrategyId, NodeId};
use types::indicator::Indicator;
use crate::response::{Response, ResponseTrait};
use types::cache::CacheKey;

#[derive(Debug)]
pub enum IndicatorEngineResponse {
    RegisterIndicator(RegisterIndicatorResponse),
    CalculateBacktestIndicator(CalculateBacktestIndicatorResponse),
}

impl ResponseTrait for IndicatorEngineResponse {
    fn code(&self) -> i32 {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.code,
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => response.code,
        }
    }

    fn message(&self) -> String {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.message.clone(),
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => response.message.clone(),
        }
    }

    fn response_timestamp(&self) -> i64 {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.response_timestamp,
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => response.response_timestamp,
        }
    }
}

impl From<IndicatorEngineResponse> for Response {
    fn from(response: IndicatorEngineResponse) -> Self {
        Response::IndicatorEngine(response)
    }
}


#[derive(Debug)]
pub struct CalculateBacktestIndicatorResponse {
    pub code: i32,
    pub message: String,
    pub backtest_indicator_cache_key: CacheKey,
    pub response_timestamp: i64,
}

#[derive(Debug)]
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
}
