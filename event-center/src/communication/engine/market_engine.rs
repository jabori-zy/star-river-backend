use super::{EngineResponse, GenericEngineCommand};
use derive_more::From;
use star_river_core::custom_type::{AccountId, StrategyId};
use star_river_core::market::{Exchange, Kline, KlineInterval, Symbol};
use star_river_core::strategy::TimeRange;

#[derive(Debug, From)]
pub enum MarketEngineCommand {
    SubscribeKlineStream(SubscribeKlineStreamCommand),
    UnsubscribeKlineStream(UnsubscribeKlineStreamCommand),
    GetKlineHistory(GetKlineHistoryCommand),
    GetSymbolInfo(GetSymbolInfoCommand),
}

// ============ Command and Response Type Definitions ============
pub type SubscribeKlineStreamCommand = GenericEngineCommand<SubscribeKlineStreamCmdPayload, SubscribeKlineStreamRespPayload>;
pub type SubscribeKlineStreamResponse = EngineResponse<SubscribeKlineStreamRespPayload>;

pub type UnsubscribeKlineStreamCommand = GenericEngineCommand<UnsubscribeKlineStreamCmdPayload, UnsubscribeKlineStreamRespPayload>;
pub type UnsubscribeKlineStreamResponse = EngineResponse<UnsubscribeKlineStreamRespPayload>;

pub type GetKlineHistoryCommand = GenericEngineCommand<GetKlineHistoryCmdPayload, GetKlineHistoryRespPayload>;
pub type GetKlineHistoryResponse = EngineResponse<GetKlineHistoryRespPayload>;

pub type GetSymbolInfoCommand = GenericEngineCommand<GetSymbolInfoCmdPayload, GetSymbolInfoRespPayload>;
pub type GetSymbolInfoResponse = EngineResponse<GetSymbolInfoRespPayload>;

// ============ Subscribe Kline Stream Command ============
#[derive(Debug)]
pub struct SubscribeKlineStreamCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub cache_size: u32,
}

#[derive(Debug)]
pub struct SubscribeKlineStreamRespPayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl SubscribeKlineStreamRespPayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            exchange,
            symbol,
            interval,
        }
    }
}

// ============ Unsubscribe Kline Stream Command ============
#[derive(Debug)]
pub struct UnsubscribeKlineStreamCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
}

#[derive(Debug)]
pub struct UnsubscribeKlineStreamRespPayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
}

impl UnsubscribeKlineStreamRespPayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        Self {
            exchange,
            symbol,
            interval,
        }
    }
}

// ============ Get Kline History Command ============
#[derive(Debug)]
pub struct GetKlineHistoryCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: String,
    pub account_id: AccountId,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub time_range: TimeRange,
}

impl GetKlineHistoryCmdPayload {
    pub fn new(
        strategy_id: StrategyId,
        node_id: String,
        account_id: AccountId,
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Self {
        Self {
            strategy_id,
            node_id,
            account_id,
            exchange,
            symbol,
            interval,
            time_range,
        }
    }
}

#[derive(Debug)]
pub struct GetKlineHistoryRespPayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub kline_history: Vec<Kline>,
}

impl GetKlineHistoryRespPayload {
    pub fn new(exchange: Exchange, symbol: String, interval: KlineInterval, kline_history: Vec<Kline>) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            kline_history,
        }
    }
}


#[derive(Debug)]
pub struct GetSymbolInfoCmdPayload {
    pub account_id: AccountId,
    pub symbol: String,
}

impl GetSymbolInfoCmdPayload {
    pub fn new(account_id: AccountId, symbol: String) -> Self {
        Self {
            account_id,
            symbol,
        }
    }
}


#[derive(Debug)]
pub struct GetSymbolInfoRespPayload {
    pub symbol: Symbol,
}

impl GetSymbolInfoRespPayload {
    pub fn new(symbol: Symbol) -> Self {
        Self { symbol }
    }
}
