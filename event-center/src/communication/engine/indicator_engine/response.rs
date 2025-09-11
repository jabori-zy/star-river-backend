use super::super::{EngineResponse, ResponseTrait};
use chrono::{DateTime, FixedOffset};
use star_river_core::cache::Key;
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::indicator::IndicatorConfig;
use star_river_core::market::{Exchange, KlineInterval};
use std::sync::Arc;
use utils::get_utc8_datetime;

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
            IndicatorEngineResponse::RegisterIndicator(response) => {
                response.error.as_ref().unwrap().clone()
            }
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => {
                response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTime<FixedOffset> {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.datetime,
            IndicatorEngineResponse::CalculateBacktestIndicator(response) => response.datetime,
        }
    }
}

impl From<IndicatorEngineResponse> for EngineResponse {
    fn from(response: IndicatorEngineResponse) -> Self {
        EngineResponse::IndicatorEngine(response)
    }
}

#[derive(Debug)]
pub struct CalculateBacktestIndicatorResponse {
    pub success: bool,
    pub backtest_indicator_key: Key,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTime<FixedOffset>,
}

impl CalculateBacktestIndicatorResponse {
    pub fn success(backtest_indicator_key: Key) -> Self {
        Self {
            success: true,
            backtest_indicator_key,
            error: None,
            datetime: get_utc8_datetime(),
        }
    }
}

impl From<CalculateBacktestIndicatorResponse> for EngineResponse {
    fn from(response: CalculateBacktestIndicatorResponse) -> Self {
        EngineResponse::IndicatorEngine(IndicatorEngineResponse::CalculateBacktestIndicator(
            response,
        ))
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
    pub datetime: DateTime<FixedOffset>,
}

impl RegisterIndicatorResponse {
    pub fn success(
        strategy_id: StrategyId,
        node_id: NodeId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        indicator: IndicatorConfig,
    ) -> Self {
        Self {
            success: true,
            strategy_id,
            node_id,
            exchange,
            symbol,
            interval,
            indicator,
            error: None,
            datetime: get_utc8_datetime(),
        }
    }
}

impl From<RegisterIndicatorResponse> for EngineResponse {
    fn from(response: RegisterIndicatorResponse) -> Self {
        EngineResponse::IndicatorEngine(IndicatorEngineResponse::RegisterIndicator(response))
    }
}
