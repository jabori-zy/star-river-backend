// mod command;
// mod response;

use std::collections::HashMap;
use std::sync::Arc;

use super::{GenericEngineCommand, EngineResponse};

// pub use command::*;
// pub use response::*;



use derive_more::From;
use star_river_core::cache::{CacheValue, Key};
use star_river_core::custom_type::{NodeId, StrategyId};
use tokio::time::Duration;



#[derive(Debug, From)]
pub enum CacheEngineCommand {
    AddKey(AddKeyCommand),
    GetCache(GetCacheCommand),
    GetCacheMulti(GetCacheMultiCommand),
    GetCacheLength(GetCacheLengthCommand),
    GetCacheLengthMulti(GetCacheLengthMultiCommand),
    UpdateCache(UpdateCacheCommand),
    ClearCache(ClearCacheCommand),
}



pub type AddKeyCommand = GenericEngineCommand<AddKeyCmdPayload, AddKeyRespPayload>;
pub type AddKeyResponse = EngineResponse<AddKeyRespPayload>;

pub type GetCacheCommand = GenericEngineCommand<GetCacheCmdPayload, GetCacheRespPayload>;
pub type GetCacheResponse = EngineResponse<GetCacheRespPayload>;

pub type GetCacheMultiCommand = GenericEngineCommand<GetCacheMultiCmdPayload, GetCacheMultiRespPayload>;
pub type GetCacheMultiResponse = EngineResponse<GetCacheMultiRespPayload>;

pub type GetCacheLengthCommand = GenericEngineCommand<GetCacheLengthCmdPayload, GetCacheLengthRespPayload>;
pub type GetCacheLengthResponse = EngineResponse<GetCacheLengthRespPayload>;

pub type GetCacheLengthMultiCommand = GenericEngineCommand<GetCacheLengthMultiCmdPayload, GetCacheLengthMultiRespPayload>;
pub type GetCacheLengthMultiResponse = EngineResponse<GetCacheLengthMultiRespPayload>;

pub type UpdateCacheCommand = GenericEngineCommand<UpdateCacheCmdPayload, UpdateCacheRespPayload>;
pub type UpdateCacheResponse = EngineResponse<UpdateCacheRespPayload>;

pub type ClearCacheCommand = GenericEngineCommand<ClearCacheCmdPayload, ClearCacheRespPayload>;
pub type ClearCacheResponse = EngineResponse<ClearCacheRespPayload>;


// ============ Add Key Command ============
#[derive(Debug)]
pub struct AddKeyCmdPayload {
    pub strategy_id: StrategyId,
    pub key: Key,
    pub max_size: Option<u32>,
    pub duration: Duration,
}

impl AddKeyCmdPayload {
    pub fn new(strategy_id: StrategyId, key: Key, max_size: Option<u32>, duration: Duration) -> Self {
        Self { strategy_id, key, max_size, duration }
    }
}


#[derive(Debug)]
pub struct AddKeyRespPayload {
    pub key: Key
}

impl AddKeyRespPayload {
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}


// ============ Get Cache Command ============
#[derive(Debug)]
pub struct GetCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub node_id: NodeId,
    pub key: Key,
    pub index: Option<u32>,
    pub limit: Option<u32>,
}

impl GetCacheCmdPayload {
    pub fn new(strategy_id: StrategyId, node_id: NodeId, key: Key, index: Option<u32>, limit: Option<u32>) -> Self {
        Self { strategy_id, node_id, key, index, limit }
    }
}

#[derive(Debug)]
pub struct GetCacheRespPayload {
    pub data: Vec<Arc<CacheValue>>,
}

impl GetCacheRespPayload {
    pub fn new(data: Vec<Arc<CacheValue>>) -> Self {
        Self { data }
    }
}


// ============ Get Cache Multi Command ============
#[derive(Debug)]
pub struct GetCacheMultiCmdPayload {
    pub strategy_id: StrategyId,
    pub keys: Vec<Key>,
    pub index: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug)]
pub struct GetCacheMultiRespPayload {
    pub data: HashMap<String, Vec<Vec<f64>>>,
}

impl GetCacheMultiRespPayload {
    pub fn new(data: HashMap<String, Vec<Vec<f64>>>) -> Self {
        Self { data }
    }
}

// ============ Get Cache Length Command ============
#[derive(Debug)]
pub struct GetCacheLengthCmdPayload {
    pub strategy_id: StrategyId,
    pub key: Key,
}

#[derive(Debug)]
pub struct GetCacheLengthRespPayload {
    pub key: Key,
    pub length: u32,
}


// ============ Get Cache Length Multi Command ============
#[derive(Debug)]
pub struct GetCacheLengthMultiCmdPayload {
    pub strategy_id: StrategyId,
    pub keys: Vec<Key>,
}

impl GetCacheLengthMultiCmdPayload {
    pub fn new(strategy_id: StrategyId, keys: Vec<Key>) -> Self {
        Self { strategy_id, keys }
    }
}

#[derive(Debug)]
pub struct GetCacheLengthMultiRespPayload {
    pub keys: Vec<Key>,
    pub lengths: HashMap<Key, u32>,
}

impl GetCacheLengthMultiRespPayload {
    pub fn new(keys: Vec<Key>, lengths: HashMap<Key, u32>) -> Self {
        Self { keys, lengths }
    }
}


// ============ Update Cache Command ============
#[derive(Debug)]
pub struct UpdateCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub key: Key,
    pub value: Arc<CacheValue>,
}

#[derive(Debug)]
pub struct UpdateCacheRespPayload {
    pub key: Key,
}

impl UpdateCacheRespPayload {
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}




// ============ Clear Cache Command ============
#[derive(Debug)]
pub struct ClearCacheCmdPayload {
    pub strategy_id: StrategyId,
    pub key: Key,
}

impl ClearCacheCmdPayload {
    pub fn new(strategy_id: StrategyId, key: Key) -> Self {
        Self { strategy_id, key }
    }
}

#[derive(Debug)]
pub struct ClearCacheRespPayload {
    pub key: Key,
}

impl ClearCacheRespPayload {
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}