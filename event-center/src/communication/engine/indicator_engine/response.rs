use super::super::{EngineResponse, ResponseTrait};
use chrono::Utc;
use star_river_core::cache::key::{IndicatorKey, KlineKey};
use star_river_core::cache::Key;
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::IndicatorConfig;
use star_river_core::market::{Exchange, KlineInterval};
use star_river_core::system::DateTimeUtc;
use std::sync::Arc;

#[derive(Debug)]
pub enum IndicatorEngineResponse {
    RegisterIndicator(RegisterIndicatorResponse),
    CalculateHistoryIndicator(CalculateHistoryIndicatorResponse),
    CalculateIndicator(CalculateIndicatorResponse),
    GetIndicatorLookback(GetIndicatorLookbackResponse),
}

impl ResponseTrait for IndicatorEngineResponse {
    fn success(&self) -> bool {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.success,
            IndicatorEngineResponse::CalculateHistoryIndicator(response) => response.success,
            IndicatorEngineResponse::CalculateIndicator(response) => response.success,
            IndicatorEngineResponse::GetIndicatorLookback(response) => response.success,
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.error.as_ref().unwrap().clone(),
            IndicatorEngineResponse::CalculateHistoryIndicator(response) => response.error.as_ref().unwrap().clone(),
            IndicatorEngineResponse::CalculateIndicator(response) => response.error.as_ref().unwrap().clone(),
            IndicatorEngineResponse::GetIndicatorLookback(response) => response.error.as_ref().unwrap().clone(),
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            IndicatorEngineResponse::RegisterIndicator(response) => response.datetime,
            IndicatorEngineResponse::CalculateHistoryIndicator(response) => response.datetime,
            IndicatorEngineResponse::CalculateIndicator(response) => response.datetime,
            IndicatorEngineResponse::GetIndicatorLookback(response) => response.datetime,
        }
    }
}

impl From<IndicatorEngineResponse> for EngineResponse {
    fn from(response: IndicatorEngineResponse) -> Self {
        EngineResponse::IndicatorEngine(response)
    }
}

#[derive(Debug)]
pub struct CalculateHistoryIndicatorResponse {
    pub success: bool,
    pub kline_key: KlineKey,
    pub indicator_config: IndicatorConfig,
    pub indicators: Vec<Indicator>,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl CalculateHistoryIndicatorResponse {
    pub fn success(kline_key: KlineKey, indicator_config: IndicatorConfig, indicators: Vec<Indicator>) -> Self {
        Self {
            success: true,
            kline_key,
            indicator_config,
            indicators,
            error: None,
            datetime: Utc::now(),
        }
    }

    pub fn error(error: Arc<dyn StarRiverErrorTrait>, kline_key: KlineKey, indicator_config: IndicatorConfig) -> Self {
        Self {
            success: false,
            kline_key,
            indicator_config,
            indicators: Vec::new(),
            error: Some(error),
            datetime: Utc::now(),
        }
    }
}

impl From<CalculateHistoryIndicatorResponse> for EngineResponse {
    fn from(response: CalculateHistoryIndicatorResponse) -> Self {
        EngineResponse::IndicatorEngine(IndicatorEngineResponse::CalculateHistoryIndicator(response))
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
    pub datetime: DateTimeUtc,
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
            datetime: Utc::now(),
        }
    }
}

impl From<RegisterIndicatorResponse> for EngineResponse {
    fn from(response: RegisterIndicatorResponse) -> Self {
        EngineResponse::IndicatorEngine(IndicatorEngineResponse::RegisterIndicator(response))
    }
}

#[derive(Debug)]
pub struct CalculateIndicatorResponse {
    pub success: bool,
    pub indicator_key: Key,
    pub indicator: Option<Indicator>,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl CalculateIndicatorResponse {
    pub fn success(indicator_key: Key, indicator: Indicator) -> Self {
        Self {
            success: true,
            indicator_key,
            indicator: Some(indicator),
            error: None,
            datetime: Utc::now(),
        }
    }

    pub fn error(error: Arc<dyn StarRiverErrorTrait>, indicator_key: Key) -> Self {
        Self {
            success: false,
            indicator_key,
            indicator: None,
            error: Some(error),
            datetime: Utc::now(),
        }
    }
}

impl From<CalculateIndicatorResponse> for EngineResponse {
    fn from(response: CalculateIndicatorResponse) -> Self {
        EngineResponse::IndicatorEngine(IndicatorEngineResponse::CalculateIndicator(response))
    }
}

#[derive(Debug)]
pub struct GetIndicatorLookbackResponse {
    pub success: bool,
    pub indicator_key: IndicatorKey,
    pub lookback: usize,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl GetIndicatorLookbackResponse {
    pub fn success(indicator_key: IndicatorKey, lookback: usize) -> Self {
        Self {
            success: true,
            indicator_key,
            lookback,
            error: None,
            datetime: Utc::now(),
        }
    }
}

impl From<GetIndicatorLookbackResponse> for EngineResponse {
    fn from(response: GetIndicatorLookbackResponse) -> Self {
        EngineResponse::IndicatorEngine(IndicatorEngineResponse::GetIndicatorLookback(response))
    }
}