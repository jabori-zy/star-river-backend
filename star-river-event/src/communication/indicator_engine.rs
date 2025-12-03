use derive_more::From;
use event_center_core::communication::{Command, Response};
use key::{IndicatorKey, KlineKey};
use star_river_core::{
    custom_type::{NodeId, StrategyId},
    kline::Kline,
};
use ta_lib::{Indicator, IndicatorConfig};

// ============ Indicator Engine Command Enum ============
#[derive(Debug, From)]
pub enum IndicatorEngineCommand {
    CalculateIndicator(CalculateIndicatorCommand),
    CalculateLookback(CalculateLookbackCommand),
}

// ============ Command and Response Type Definitions ============
pub type CalculateIndicatorCommand = Command<CalculateIndicatorCmdPayload, CalculateRespPayload>;
pub type CalculateIndicatorResponse = Response<CalculateRespPayload>;

pub type CalculateLookbackCommand = Command<CalculateLookbackCmdPayload, CalculateLookbackRespPayload>;
pub type CalculateLookbackResponse = Response<CalculateLookbackRespPayload>;

// ============ Calculate History Indicator Command ============
#[derive(Debug)]
pub struct CalculateIndicatorCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub kline_series: Vec<Kline>,
    pub indicator_config: IndicatorConfig,
}

impl CalculateIndicatorCmdPayload {
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
pub struct CalculateRespPayload {
    pub kline_key: KlineKey,
    pub indicator_config: IndicatorConfig,
    pub indicators: Vec<Indicator>,
}

impl CalculateRespPayload {
    pub fn new(kline_key: KlineKey, indicator_config: IndicatorConfig, indicators: Vec<Indicator>) -> Self {
        Self {
            kline_key,
            indicator_config,
            indicators,
        }
    }
}

// ============ Get Indicator Lookback Command ============
#[derive(Debug)]
pub struct CalculateLookbackCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub indicator_key: IndicatorKey,
}

impl CalculateLookbackCmdPayload {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, indicator_key: IndicatorKey) -> Self {
        Self {
            strategy_id,
            node_id,
            indicator_key,
        }
    }
}

#[derive(Debug)]
pub struct CalculateLookbackRespPayload {
    pub indicator_key: IndicatorKey,
    pub lookback: usize,
}

impl CalculateLookbackRespPayload {
    pub fn new(indicator_key: IndicatorKey, lookback: usize) -> Self {
        Self { indicator_key, lookback }
    }
}
