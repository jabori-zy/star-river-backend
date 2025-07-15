use serde::{Deserialize, Serialize};
use strum::Display;
use types::custom_type::{StrategyId, NodeId};
use std::fmt::Debug;
use types::market::{Exchange, KlineInterval};
use uuid::Uuid;
use types::indicator::IndicatorConfig;
use types::cache::key::BacktestKlineKey;
use crate::command::Command;
use crate::Responder;
use super::CommandTrait;

#[derive(Debug)]
pub enum IndicatorEngineCommand {
    RegisterIndicator(RegisterIndicatorParams), // 注册指标
    CalculateBacktestIndicator(CalculateBacktestIndicatorParams), // 计算回测指标
}

impl CommandTrait for IndicatorEngineCommand {
    fn responder(&self) -> &Responder {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => &params.responder,
            IndicatorEngineCommand::CalculateBacktestIndicator(params) => &params.responder,
        }
    }
    fn timestamp(&self) -> i64 {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => params.command_timestamp,
            IndicatorEngineCommand::CalculateBacktestIndicator(params) => params.command_timestamp,
        }
    }

    fn sender(&self) -> String {
        match self {
            IndicatorEngineCommand::RegisterIndicator(params) => params.sender.clone(),
            IndicatorEngineCommand::CalculateBacktestIndicator(params) => params.sender.clone(),
        }
    }
    
}

impl From<IndicatorEngineCommand> for Command {
    fn from(command: IndicatorEngineCommand) -> Self {
        Command::IndicatorEngine(command)
    }
}


#[derive(Debug)]
pub struct RegisterIndicatorParams {
    pub strategy_id: StrategyId, // 策略ID
    pub node_id: NodeId, // 节点ID
    pub exchange: Exchange, // 交易所
    pub symbol: String, // 符号
    pub interval: KlineInterval, // 时间周期
    pub indicator_config: IndicatorConfig, // 指标配置
    pub sender: String, // 发送者
    pub command_timestamp:i64, // 命令时间戳
    pub responder: Responder, // 响应者
}

#[derive(Debug)]
// 计算回测指标命令参数
pub struct CalculateBacktestIndicatorParams {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub kline_cache_key: BacktestKlineKey, // 回测K线缓存键
    pub indicator_config: IndicatorConfig,
    pub sender: String,
    pub command_timestamp:i64,
    pub responder: Responder,
}