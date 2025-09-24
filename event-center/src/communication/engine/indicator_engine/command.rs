use super::super::{EngineCommand, EngineCommandTrait, EngineResponder};
use chrono::Utc;
use star_river_core::cache::key::{IndicatorKey, KlineKey};
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::indicator::IndicatorConfig;
use star_river_core::market::{Exchange, Kline, KlineInterval};
use std::fmt::Debug;

use star_river_core::system::DateTimeUtc;

#[derive(Debug)]
pub enum IndicatorEngineCommand {
    RegisterIndicator(RegisterIndicatorParams),                 // 注册指标
    CalculateHistoryIndicator(CalculateHistoryIndicatorParams), // 计算历史指标
    CalculateIndicator(CalculateIndicatorParams),               // 计算指标
    GetIndicatorLookback(GetIndicatorLookbackParams),           // 获取指标lookback
}

impl EngineCommandTrait for IndicatorEngineCommand {
    fn responder(&self) -> &EngineResponder {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => &params.responder,
            IndicatorEngineCommand::CalculateHistoryIndicator(params) => &params.responder,
            IndicatorEngineCommand::CalculateIndicator(params) => &params.responder,
            IndicatorEngineCommand::GetIndicatorLookback(params) => &params.responder,
        }
    }
    fn datetime(&self) -> DateTimeUtc {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => params.datetime,
            IndicatorEngineCommand::CalculateHistoryIndicator(params) => params.datetime,
            IndicatorEngineCommand::CalculateIndicator(params) => params.datetime,
            IndicatorEngineCommand::GetIndicatorLookback(params) => params.datetime,
        }
    }

    fn sender(&self) -> String {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => params.sender.clone(),
            IndicatorEngineCommand::CalculateHistoryIndicator(params) => params.sender.clone(),
            IndicatorEngineCommand::CalculateIndicator(params) => params.sender.clone(),
            IndicatorEngineCommand::GetIndicatorLookback(params) => params.sender.clone(),
        }
    }
}

impl From<IndicatorEngineCommand> for EngineCommand {
    fn from(command: IndicatorEngineCommand) -> Self {
        EngineCommand::IndicatorEngine(command)
    }
}

#[derive(Debug)]
pub struct RegisterIndicatorParams {
    pub strategy_id: StrategyId,           // 策略ID
    pub node_id: NodeId,                   // 节点ID
    pub exchange: Exchange,                // 交易所
    pub symbol: String,                    // 符号
    pub interval: KlineInterval,           // 时间周期
    pub indicator_config: IndicatorConfig, // 指标配置
    pub sender: String,                    // 发送者
    pub datetime: DateTimeUtc,             // 命令时间戳
    pub responder: EngineResponder,        // 响应者
}

impl RegisterIndicatorParams {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        indicator_config: IndicatorConfig,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            exchange,
            symbol,
            interval,
            indicator_config,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
// 计算回测指标命令参数
pub struct CalculateHistoryIndicatorParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_key: KlineKey, // 回测K线缓存键
    pub kline_series: Vec<Kline>, // 回测K线
    pub indicator_config: IndicatorConfig,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl CalculateHistoryIndicatorParams {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        kline_key: KlineKey,
        kline_series: Vec<Kline>,
        indicator_config: IndicatorConfig,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            kline_key,
            kline_series,
            indicator_config,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<CalculateHistoryIndicatorParams> for EngineCommand {
    fn from(params: CalculateHistoryIndicatorParams) -> Self {
        EngineCommand::IndicatorEngine(IndicatorEngineCommand::CalculateHistoryIndicator(params))
    }
}

#[derive(Debug)]
pub struct CalculateIndicatorParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub indicator_config: IndicatorConfig,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl CalculateIndicatorParams {
    pub fn new(
        strategy_id: StrategyId,
        node_id: NodeId,
        kline_key: KlineKey,
        indicator_config: IndicatorConfig,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            kline_key,
            indicator_config,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<CalculateIndicatorParams> for EngineCommand {
    fn from(params: CalculateIndicatorParams) -> Self {
        EngineCommand::IndicatorEngine(IndicatorEngineCommand::CalculateIndicator(params))
    }
}



#[derive(Debug)]
pub struct GetIndicatorLookbackParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub indicator_key: IndicatorKey,
    pub sender: String,
    pub datetime: DateTimeUtc,
    pub responder: EngineResponder,
}

impl GetIndicatorLookbackParams {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, indicator_key: IndicatorKey, sender: String, responder: EngineResponder) -> Self {
        Self {
            strategy_id,
            node_id,
            indicator_key,
            sender,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetIndicatorLookbackParams> for EngineCommand {
    fn from(params: GetIndicatorLookbackParams) -> Self {
        EngineCommand::IndicatorEngine(IndicatorEngineCommand::GetIndicatorLookback(params))
    }
}
