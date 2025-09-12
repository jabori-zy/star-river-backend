use super::super::{EngineCommand, EngineCommandTrait, EngineResponder};
use chrono::{DateTime, FixedOffset};
use star_river_core::cache::key::KlineKey;
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::indicator::IndicatorConfig;
use star_river_core::market::{Exchange, KlineInterval};
use std::fmt::Debug;
use star_river_core::utils::get_utc8_datetime;

#[derive(Debug)]
pub enum IndicatorEngineCommand {
    RegisterIndicator(RegisterIndicatorParams), // 注册指标
    CalculateBacktestIndicator(CalculateBacktestIndicatorParams), // 计算回测指标
}

impl EngineCommandTrait for IndicatorEngineCommand {
    fn responder(&self) -> &EngineResponder {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => &params.responder,
            IndicatorEngineCommand::CalculateBacktestIndicator(params) => &params.responder,
        }
    }
    fn datetime(&self) -> DateTime<FixedOffset> {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => params.datetime,
            IndicatorEngineCommand::CalculateBacktestIndicator(params) => params.datetime,
        }
    }

    fn sender(&self) -> String {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => params.sender.clone(),
            IndicatorEngineCommand::CalculateBacktestIndicator(params) => params.sender.clone(),
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
    pub datetime: DateTime<FixedOffset>,   // 命令时间戳
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
            datetime: get_utc8_datetime(),
            responder,
        }
    }
}

#[derive(Debug)]
// 计算回测指标命令参数
pub struct CalculateBacktestIndicatorParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_key: KlineKey, // 回测K线缓存键
    pub indicator_config: IndicatorConfig,
    pub sender: String,
    pub datetime: DateTime<FixedOffset>,
    pub responder: EngineResponder,
}

impl CalculateBacktestIndicatorParams {
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
            datetime: get_utc8_datetime(),
            responder,
        }
    }
}

impl From<CalculateBacktestIndicatorParams> for EngineCommand {
    fn from(params: CalculateBacktestIndicatorParams) -> Self {
        EngineCommand::IndicatorEngine(IndicatorEngineCommand::CalculateBacktestIndicator(params))
    }
}
