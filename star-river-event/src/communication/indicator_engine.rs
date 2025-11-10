use derive_more::From;
use event_center_core::communication::{Command, Response};
use key::{IndicatorKey, Key, KlineKey};
use star_river_core::{
    custom_type::{NodeId, StrategyId},
    exchange::Exchange,
    kline::{Kline, KlineInterval},
};
use ta_lib::{Indicator, IndicatorConfig};

// ============ Indicator Engine Command Enum ============
#[derive(Debug, From)]
pub enum IndicatorEngineCommand {
    RegisterIndicator(RegisterIndicatorCommand),
    CalculateHistoryIndicator(CalculateHistoryIndicatorCommand),
    CalculateIndicator(CalculateIndicatorCommand),
    GetIndicatorLookback(GetIndicatorLookbackCommand),
}

// ============ Command and Response Type Definitions ============
pub type RegisterIndicatorCommand = Command<RegisterIndicatorCmdPayload, RegisterIndicatorRespPayload>;
pub type RegisterIndicatorResponse = Response<RegisterIndicatorRespPayload>;

pub type CalculateHistoryIndicatorCommand = Command<CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorRespPayload>;
pub type CalculateHistoryIndicatorResponse = Response<CalculateHistoryIndicatorRespPayload>;

pub type CalculateIndicatorCommand = Command<CalculateIndicatorCmdPayload, CalculateIndicatorRespPayload>;
pub type CalculateIndicatorResponse = Response<CalculateIndicatorRespPayload>;

pub type GetIndicatorLookbackCommand = Command<GetIndicatorLookbackCmdPayload, GetIndicatorLookbackRespPayload>;
pub type GetIndicatorLookbackResponse = Response<GetIndicatorLookbackRespPayload>;

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

impl RegisterIndicatorCmdPayload {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        indicator_config: IndicatorConfig,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            exchange,
            symbol,
            interval,
            indicator_config,
        }
    }
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

impl RegisterIndicatorRespPayload {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        indicator: IndicatorConfig,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            exchange,
            symbol,
            interval,
            indicator,
        }
    }
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

impl CalculateIndicatorCmdPayload {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, kline_key: KlineKey, indicator_config: IndicatorConfig) -> Self {
        Self {
            strategy_id,
            node_id,
            kline_key,
            indicator_config,
        }
    }
}

#[derive(Debug)]
pub struct CalculateIndicatorRespPayload {
    pub indicator_key: Key,
    pub indicator: Option<Indicator>,
}

impl CalculateIndicatorRespPayload {
    pub fn new(indicator_key: Key, indicator: Option<Indicator>) -> Self {
        Self { indicator_key, indicator }
    }
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
