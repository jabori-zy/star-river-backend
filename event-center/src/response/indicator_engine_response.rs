
use types::market::{Exchange, KlineInterval};
use types::indicator::IndicatorConfig;
use types::custom_type::{StrategyId, NodeId};
use crate::response::{Response, ResponseTrait};
use types::cache::Key;
use utils::get_utc8_timestamp;
use std::sync::Arc;
use types::error::error_trait::StarRiverErrorTrait;

#[derive(Debug)]
pub enum IndicatorEngineResponse {
    RegisterIndicator(RegisterIndicatorResponse),
    CalculateBacktestIndicator(CalculateBacktestIndicatorResponse),
}

impl ResponseTrait for IndicatorEngineResponse {
    fn success(&self) -> bool {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.success,
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => response.success,
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.error.as_ref().unwrap().clone(),
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => response.error.as_ref().unwrap().clone(),
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
    pub success: bool,
    pub backtest_indicator_key: Key,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub response_timestamp: i64,
}


impl CalculateBacktestIndicatorResponse {
    pub fn success(backtest_indicator_key: Key) -> Self {
        Self {
            success: true,
            backtest_indicator_key,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<CalculateBacktestIndicatorResponse> for Response {
    fn from(response: CalculateBacktestIndicatorResponse) -> Self {
        Response::IndicatorEngine(IndicatorEngineResponse::CalculateBacktestIndicator(response))
    }
}


#[derive(Debug)]
pub struct RegisterIndicatorResponse {
    pub success: bool,
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub response_timestamp: i64,
}

impl RegisterIndicatorResponse {
    pub fn success(strategy_id: StrategyId, node_id: NodeId, exchange: Exchange, symbol: String, interval: KlineInterval, indicator: IndicatorConfig) -> Self {
        Self {
            success: true,
            strategy_id,
            node_id,
            exchange,
            symbol,
            interval,
            indicator,
            error: None,
            response_timestamp: get_utc8_timestamp(),
        }
    }
}

impl From<RegisterIndicatorResponse> for Response {
    fn from(response: RegisterIndicatorResponse) -> Self {
        Response::IndicatorEngine(IndicatorEngineResponse::RegisterIndicator(response))
    }
}