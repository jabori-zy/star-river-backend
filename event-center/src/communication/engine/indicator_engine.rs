use super::{EngineResponse, GenericEngineCommand};

use derive_more::From;
use star_river_core::custom_type::{NodeId, StrategyId};
use ta_lib::{Indicator, IndicatorConfig};
use star_river_core::key::Key;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::market::{Exchange, Kline, KlineInterval};

#[derive(Debug, From)]
pub enum IndicatorEngineCommand {
    RegisterIndicator(RegisterIndicatorCommand),
    CalculateHistoryIndicator(CalculateHistoryIndicatorCommand),
    CalculateIndicator(CalculateIndicatorCommand),
    GetIndicatorLookback(GetIndicatorLookbackCommand),
}

// ============ Command and Response Type Definitions ============
pub type RegisterIndicatorCommand = GenericEngineCommand<RegisterIndicatorCmdPayload, RegisterIndicatorRespPayload>;
pub type RegisterIndicatorResponse = EngineResponse<RegisterIndicatorRespPayload>;

pub type CalculateHistoryIndicatorCommand = GenericEngineCommand<CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorRespPayload>;
pub type CalculateHistoryIndicatorResponse = EngineResponse<CalculateHistoryIndicatorRespPayload>;

pub type CalculateIndicatorCommand = GenericEngineCommand<CalculateIndicatorCmdPayload, CalculateIndicatorRespPayload>;
pub type CalculateIndicatorResponse = EngineResponse<CalculateIndicatorRespPayload>;

pub type GetIndicatorLookbackCommand = GenericEngineCommand<GetIndicatorLookbackCmdPayload, GetIndicatorLookbackRespPayload>;
pub type GetIndicatorLookbackResponse = EngineResponse<GetIndicatorLookbackRespPayload>;

// ============ Register Indicator Command ============
#[derive(Debug)]
pub struct RegisterIndicatorCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
}

#[derive(Debug)]
pub struct RegisterIndicatorRespPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: IndicatorConfig,
}

// ============ Calculate History Indicator Command ============
#[derive(Debug)]
pub struct CalculateHistoryIndicatorCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub kline_series: Vec<Kline>,
    pub indicator_config: IndicatorConfig,
}

impl CalculateHistoryIndicatorCmdPayload {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        kline_key: KlineKey,
        kline_series: Vec<Kline>,
        indicator_config: IndicatorConfig,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            kline_key,
            kline_series,
            indicator_config,
        }
    }
}

#[derive(Debug)]
pub struct CalculateHistoryIndicatorRespPayload {
    pub kline_key: KlineKey,
    pub indicator_config: IndicatorConfig,
    pub indicators: Vec<Indicator>,
}

impl CalculateHistoryIndicatorRespPayload {
    pub fn new(kline_key: KlineKey, indicator_config: IndicatorConfig, indicators: Vec<Indicator>) -> Self {
        Self {
            kline_key,
            indicator_config,
            indicators,
        }
    }
}

// ============ Calculate Indicator Command ============
#[derive(Debug)]
pub struct CalculateIndicatorCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub indicator_config: IndicatorConfig,
}

#[derive(Debug)]
pub struct CalculateIndicatorRespPayload {
    pub indicator_key: Key,
    pub indicator: Option<Indicator>,
}

// ============ Get Indicator Lookback Command ============
#[derive(Debug)]
pub struct GetIndicatorLookbackCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub indicator_key: IndicatorKey,
}

impl GetIndicatorLookbackCmdPayload {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, indicator_key: IndicatorKey) -> Self {
        Self {
            strategy_id,
            node_id,
            indicator_key,
        }
    }
}

#[derive(Debug)]
pub struct GetIndicatorLookbackRespPayload {
    pub indicator_key: IndicatorKey,
    pub lookback: usize,
}

impl GetIndicatorLookbackRespPayload {
    pub fn new(indicator_key: IndicatorKey, lookback: usize) -> Self {
        Self { indicator_key, lookback }
    }
}
