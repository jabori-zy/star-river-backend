
use std::collections::HashMap;
use super::{GenericEngineCommand, EngineResponse};
use derive_more::From;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::custom_type::{NodeId, StrategyId};
use star_river_core::indicator::Indicator;
use star_river_core::market::Kline;
use tokio::time::Duration;



#[derive(Debug, From)]
pub enum CacheEngineCommand {
    // kline cache
    AddKlineKey(AddKlineKeyCommand),
    GetKlineCache(GetKlineCacheCommand),
    GetKlineCacheMulti(GetKlineCacheMultiCommand),
    GetKlineCacheLength(GetKlineCacheLengthCommand),
    GetKlineCacheLengthMulti(GetKlineCacheLengthMultiCommand),
    UpdateKlineCache(UpdateKlineCacheCommand),
    ClearKlineCache(ClearKlineCacheCommand),
    // indicator cache
    AddIndicatorKey(AddIndicatorKeyCommand),
    GetIndicatorCache(GetIndicatorCacheCommand),
    GetIndicatorCacheMulti(GetIndicatorCacheMultiCommand),
    GetIndicatorCacheLength(GetIndicatorCacheLengthCommand),
    GetIndicatorCacheLengthMulti(GetIndicatorCacheLengthMultiCommand),
    UpdateIndicatorCache(UpdateIndicatorCacheCommand),
    ClearIndicatorCache(ClearIndicatorCacheCommand),
}


// ============ Kline Cache Command ============
pub type AddKlineKeyCommand = GenericEngineCommand<AddKlineKeyCmdPayload, AddKlineKeyRespPayload>;
pub type AddKlineKeyResponse = EngineResponse<AddKlineKeyRespPayload>;

// ============ Get Kline Cache Command ============
pub type GetKlineCacheCommand = GenericEngineCommand<GetKlineCacheCmdPayload, GetKlineCacheRespPayload>;
pub type GetKlineCacheResponse = EngineResponse<GetKlineCacheRespPayload>;

// ============ Get Kline Cache Multi Command ============
pub type GetKlineCacheMultiCommand = GenericEngineCommand<GetKlineCacheMultiCmdPayload, GetKlineCacheMultiRespPayload>;
pub type GetKlineCacheMultiResponse = EngineResponse<GetKlineCacheMultiRespPayload>;

// ============ Get Kline Cache Length Command ============
pub type GetKlineCacheLengthCommand = GenericEngineCommand<GetKlineCacheLengthCmdPayload, GetKlineCacheLengthRespPayload>;
pub type GetKlineCacheLengthResponse = EngineResponse<GetKlineCacheLengthRespPayload>;

// ============ Get Kline Cache Length Multi Command ============
pub type GetKlineCacheLengthMultiCommand = GenericEngineCommand<GetKlineCacheLengthMultiCmdPayload, GetKlineCacheLengthMultiRespPayload>;
pub type GetKlineCacheLengthMultiResponse = EngineResponse<GetKlineCacheLengthMultiRespPayload>;

// ============ Update Kline Cache Command ============
pub type UpdateKlineCacheCommand = GenericEngineCommand<UpdateKlineCacheCmdPayload, UpdateKlineCacheRespPayload>;
pub type UpdateKlineCacheResponse = EngineResponse<UpdateKlineCacheRespPayload>;

// ============ Clear Kline Cache Command ============
pub type ClearKlineCacheCommand = GenericEngineCommand<ClearKlineCacheCmdPayload, ClearKlineCacheRespPayload>;
pub type ClearKlineCacheResponse = EngineResponse<ClearKlineCacheRespPayload>;


// ============ Add Key Command ============
#[derive(Debug)]
pub struct AddKlineKeyCmdPayload {
    pub strategy_id: StrategyId,
    pub key: KlineKey,
    pub max_size: Option<u32>,
    pub duration: Duration,
}

impl AddKlineKeyCmdPayload {
    pub fn new(strategy_id: StrategyId, key: KlineKey, max_size: Option<u32>, duration: Duration) -> Self {
        Self { strategy_id, key, max_size, duration }
    }
}


#[derive(Debug)]
pub struct AddKlineKeyRespPayload {
    pub key: KlineKey
}

impl AddKlineKeyRespPayload {
    pub fn new(key: KlineKey) -> Self {
        Self { key }
    }
}


// ============ Get Cache Command ============
#[derive(Debug)]
pub struct GetKlineCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub key: KlineKey,
    pub index: Option<u32>,
    pub limit: Option<u32>,
}

impl GetKlineCacheCmdPayload {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, key: KlineKey, index: Option<u32>, limit: Option<u32>) -> Self {
        Self { strategy_id, node_id, key, index, limit }
    }
}

#[derive(Debug)]
pub struct GetKlineCacheRespPayload {
    pub data: Vec<Kline>,
}

impl GetKlineCacheRespPayload {
    pub fn new(data: Vec<Kline>) -> Self {
        Self { data }
    }
}


// ============ Get Cache Multi Command ============
#[derive(Debug)]
pub struct GetKlineCacheMultiCmdPayload {
    pub strategy_id: StrategyId,
    pub keys: Vec<KlineKey>,
    pub index: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug)]
pub struct GetKlineCacheMultiRespPayload {
    pub data: HashMap<String, Vec<Vec<f64>>>,
}

impl GetKlineCacheMultiRespPayload {
    pub fn new(data: HashMap<String, Vec<Vec<f64>>>) -> Self {
        Self { data }
    }
}

// ============ Get Cache Length Command ============
#[derive(Debug)]
pub struct GetKlineCacheLengthCmdPayload {
    pub strategy_id: StrategyId,
    pub key: KlineKey,
}

#[derive(Debug)]
pub struct GetKlineCacheLengthRespPayload {
    pub key: KlineKey,
    pub length: u32,
}


// ============ Get Cache Length Multi Command ============
#[derive(Debug)]
pub struct GetKlineCacheLengthMultiCmdPayload {
    pub strategy_id: StrategyId,
    pub keys: Vec<KlineKey>,
}

impl GetKlineCacheLengthMultiCmdPayload {
    pub fn new(strategy_id: StrategyId, keys: Vec<KlineKey>) -> Self {
        Self { strategy_id, keys }
    }
}

#[derive(Debug)]
pub struct GetKlineCacheLengthMultiRespPayload {
    pub keys: Vec<KlineKey>,
    pub lengths: HashMap<KlineKey, u32>,
}

impl GetKlineCacheLengthMultiRespPayload {
    pub fn new(keys: Vec<KlineKey>, lengths: HashMap<KlineKey, u32>) -> Self {
        Self { keys, lengths }
    }
}


// ============ Update Cache Command ============
#[derive(Debug)]
pub struct UpdateKlineCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub key: KlineKey,
    pub value: Kline,
}

#[derive(Debug)]
pub struct UpdateKlineCacheRespPayload {
    pub key: KlineKey,
}

impl UpdateKlineCacheRespPayload {
    pub fn new(key: KlineKey) -> Self {
        Self { key }
    }
}




// ============ Clear Cache Command ============
#[derive(Debug)]
pub struct ClearKlineCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub key: KlineKey,
}

impl ClearKlineCacheCmdPayload {
    pub fn new(strategy_id: StrategyId, key: KlineKey) -> Self {
        Self { strategy_id, key }
    }
}

#[derive(Debug)]
pub struct ClearKlineCacheRespPayload {
    pub key: KlineKey,
}

impl ClearKlineCacheRespPayload {
    pub fn new(key: KlineKey) -> Self {
        Self { key }
    }
}


// ============ Indicator Cache Command ============
pub type AddIndicatorKeyCommand = GenericEngineCommand<AddIndicatorKeyCmdPayload, AddIndicatorKeyRespPayload>;
pub type AddIndicatorKeyResponse = EngineResponse<AddIndicatorKeyRespPayload>;

// ============ Get Indicator Cache Command ============
pub type GetIndicatorCacheCommand = GenericEngineCommand<GetIndicatorCacheCmdPayload, GetIndicatorCacheRespPayload>;
pub type GetIndicatorCacheResponse = EngineResponse<GetIndicatorCacheRespPayload>;

// ============ Get Indicator Cache Multi Command ============
pub type GetIndicatorCacheMultiCommand = GenericEngineCommand<GetIndicatorCacheMultiCmdPayload, GetIndicatorCacheMultiRespPayload>;
pub type GetIndicatorCacheMultiResponse = EngineResponse<GetIndicatorCacheMultiRespPayload>;

// ============ Get Indicator Cache Length Command ============
pub type GetIndicatorCacheLengthCommand = GenericEngineCommand<GetIndicatorCacheLengthCmdPayload, GetIndicatorCacheLengthRespPayload>;
pub type GetIndicatorCacheLengthResponse = EngineResponse<GetIndicatorCacheLengthRespPayload>;

// ============ Get Indicator Cache Length Multi Command ============
pub type GetIndicatorCacheLengthMultiCommand = GenericEngineCommand<GetIndicatorCacheLengthMultiCmdPayload, GetIndicatorCacheLengthMultiRespPayload>;
pub type GetIndicatorCacheLengthMultiResponse = EngineResponse<GetIndicatorCacheLengthMultiRespPayload>;

// ============ Update Indicator Cache Command ============
pub type UpdateIndicatorCacheCommand = GenericEngineCommand<UpdateIndicatorCacheCmdPayload, UpdateIndicatorCacheRespPayload>;
pub type UpdateIndicatorCacheResponse = EngineResponse<UpdateIndicatorCacheRespPayload>;

// ============ Clear Indicator Cache Command ============
pub type ClearIndicatorCacheCommand = GenericEngineCommand<ClearIndicatorCacheCmdPayload, ClearIndicatorCacheRespPayload>;
pub type ClearIndicatorCacheResponse = EngineResponse<ClearIndicatorCacheRespPayload>;


// ============ Add Key Command ============
#[derive(Debug)]
pub struct AddIndicatorKeyCmdPayload {
    pub strategy_id: StrategyId,
    pub key: IndicatorKey,
    pub max_size: Option<u32>,
    pub duration: Duration,
}

impl AddIndicatorKeyCmdPayload {
    pub fn new(strategy_id: StrategyId, key: IndicatorKey, max_size: Option<u32>, duration: Duration) -> Self {
        Self { strategy_id, key, max_size, duration }
    }
}


#[derive(Debug)]
pub struct AddIndicatorKeyRespPayload {
    pub key: IndicatorKey
}

impl AddIndicatorKeyRespPayload {
    pub fn new(key: IndicatorKey) -> Self {
        Self { key }
    }
}


// ============ Get Cache Command ============
#[derive(Debug)]
pub struct GetIndicatorCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub key: IndicatorKey,
    pub index: Option<u32>,
    pub limit: Option<u32>,
}

impl GetIndicatorCacheCmdPayload {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, key: IndicatorKey, index: Option<u32>, limit: Option<u32>) -> Self {
        Self { strategy_id, node_id, key, index, limit }
    }
}

#[derive(Debug)]
pub struct GetIndicatorCacheRespPayload {
    pub data: Vec<Indicator>,
}

impl GetIndicatorCacheRespPayload {
    pub fn new(data: Vec<Indicator>) -> Self {
        Self { data }
    }
}


// ============ Get Cache Multi Command ============
#[derive(Debug)]
pub struct GetIndicatorCacheMultiCmdPayload {
    pub strategy_id: StrategyId,
    pub keys: Vec<IndicatorKey>,
    pub index: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug)]
pub struct GetIndicatorCacheMultiRespPayload {
    pub data: HashMap<String, Vec<Vec<f64>>>,
}

impl GetIndicatorCacheMultiRespPayload {
    pub fn new(data: HashMap<String, Vec<Vec<f64>>>) -> Self {
        Self { data }
    }
}

// ============ Get Cache Length Command ============
#[derive(Debug)]
pub struct GetIndicatorCacheLengthCmdPayload {
    pub strategy_id: StrategyId,
    pub key: IndicatorKey,
}

#[derive(Debug)]
pub struct GetIndicatorCacheLengthRespPayload {
    pub key: IndicatorKey,
    pub length: u32,
}


// ============ Get Cache Length Multi Command ============
#[derive(Debug)]
pub struct GetIndicatorCacheLengthMultiCmdPayload {
    pub strategy_id: StrategyId,
    pub keys: Vec<IndicatorKey>,
}

impl GetIndicatorCacheLengthMultiCmdPayload {
    pub fn new(strategy_id: StrategyId, keys: Vec<IndicatorKey>) -> Self {
        Self { strategy_id, keys }
    }
}

#[derive(Debug)]
pub struct GetIndicatorCacheLengthMultiRespPayload {
    pub keys: Vec<IndicatorKey>,
    pub lengths: HashMap<IndicatorKey, u32>,
}

impl GetIndicatorCacheLengthMultiRespPayload {
    pub fn new(keys: Vec<IndicatorKey>, lengths: HashMap<IndicatorKey, u32>) -> Self {
        Self { keys, lengths }
    }
}


// ============ Update Cache Command ============
#[derive(Debug)]
pub struct UpdateIndicatorCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub key: IndicatorKey,
    pub value: Indicator,
}

#[derive(Debug)]
pub struct UpdateIndicatorCacheRespPayload {
    pub key: IndicatorKey,
}

impl UpdateIndicatorCacheRespPayload {
    pub fn new(key: IndicatorKey) -> Self {
        Self { key }
    }
}




// ============ Clear Cache Command ============
#[derive(Debug)]
pub struct ClearIndicatorCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub key: IndicatorKey,
}

impl ClearIndicatorCacheCmdPayload {
    pub fn new(strategy_id: StrategyId, key: IndicatorKey) -> Self {
        Self { strategy_id, key }
    }
}

#[derive(Debug)]
pub struct ClearIndicatorCacheRespPayload {
    pub key: IndicatorKey,
}

impl ClearIndicatorCacheRespPayload {
    pub fn new(key: IndicatorKey) -> Self {
        Self { key }
    }
}