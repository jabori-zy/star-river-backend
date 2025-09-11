use super::super::{EngineCommand, EngineCommandTrait, EngineResponder};
use chrono::{DateTime, FixedOffset};
use star_river_core::custom_type::{AccountId, StrategyId};
use star_river_core::market::{Exchange, KlineInterval};
use star_river_core::strategy::TimeRange;
use std::fmt::Debug;
use utils::get_utc8_datetime;

#[derive(Debug)]
pub enum MarketEngineCommand {
    SubscribeKlineStream(SubscribeKlineStreamParams), // 订阅K线流
    UnsubscribeKlineStream(UnsubscribeKlineStreamParams), // 取消订阅K线流
    GetKlineHistory(GetKlineHistoryParams),           // 获取K线历史数据
}

impl EngineCommandTrait for MarketEngineCommand {
    fn responder(&self) -> &EngineResponder {
        match self {
            MarketEngineCommand::SubscribeKlineStream(params) => &params.responder,
            MarketEngineCommand::UnsubscribeKlineStream(params) => &params.responder,
            MarketEngineCommand::GetKlineHistory(params) => &params.responder,
        }
    }
    fn datetime(&self) -> DateTime<FixedOffset> {
        match self {
            MarketEngineCommand::SubscribeKlineStream(params) => params.datetime,
            MarketEngineCommand::UnsubscribeKlineStream(params) => params.datetime,
            MarketEngineCommand::GetKlineHistory(params) => params.datetime,
        }
    }
    fn sender(&self) -> String {
        match self {
            MarketEngineCommand::SubscribeKlineStream(params) => params.sender.clone(),
            MarketEngineCommand::UnsubscribeKlineStream(params) => params.sender.clone(),
            MarketEngineCommand::GetKlineHistory(params) => params.sender.clone(),
        }
    }
}

impl From<MarketEngineCommand> for EngineCommand {
    fn from(command: MarketEngineCommand) -> Self {
        EngineCommand::MarketEngine(command)
    }
}

#[derive(Debug)]
pub struct SubscribeKlineStreamParams {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub cache_size: u32,
    pub sender: String,
    pub datetime: DateTime<FixedOffset>,
    pub responder: EngineResponder,
}

#[derive(Debug)]
pub struct UnsubscribeKlineStreamParams {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub sender: String,
    pub datetime: DateTime<FixedOffset>,
    pub responder: EngineResponder,
}

#[derive(Debug)]
pub struct GetKlineHistoryParams {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub time_range: TimeRange, // 时间范围
    pub sender: String,
    pub datetime: DateTime<FixedOffset>,
    pub responder: EngineResponder,
}

impl GetKlineHistoryParams {
    pub fn new(
        strategy_id: StrategyId,
        node_id: String,
        account_id: AccountId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        time_range: TimeRange,
        sender: String,
        responder: EngineResponder,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            account_id,
            exchange,
            symbol,
            interval,
            time_range,
            sender,
            datetime: get_utc8_datetime(),
            responder,
        }
    }
}

impl From<GetKlineHistoryParams> for EngineCommand {
    fn from(params: GetKlineHistoryParams) -> Self {
        EngineCommand::MarketEngine(MarketEngineCommand::GetKlineHistory(params))
    }
}
