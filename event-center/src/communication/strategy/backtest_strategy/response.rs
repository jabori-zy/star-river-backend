use super::super::{NodeResponse, NodeResponseTrait, StrategyResponse, StrategyResponseTrait};
use chrono::{DateTime, Utc};
use star_river_core::cache::Key;
use star_river_core::custom_type::NodeId;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::strategy::BacktestStrategyConfig;
use std::sync::Arc;
use star_river_core::system::DateTimeUtc;


#[derive(Debug)]
pub enum BacktestStrategyResponse {
    GetStartNodeConfig(GetStartNodeConfigResponse),
    NodeReset(NodeResetResponse),
}

impl StrategyResponseTrait for BacktestStrategyResponse {
    fn success(&self) -> bool {
        match self {
            BacktestStrategyResponse::GetStartNodeConfig(get_start_node_config_response) => {
                get_start_node_config_response.success
            }
            BacktestStrategyResponse::NodeReset(node_reset_response) => node_reset_response.success,
        }
    }

    fn node_id(&self) -> &NodeId {
        match self {
            BacktestStrategyResponse::GetStartNodeConfig(get_start_node_config_response) => {
                &get_start_node_config_response.node_id
            }
            BacktestStrategyResponse::NodeReset(node_reset_response) => {
                &node_reset_response.node_id
            }
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            BacktestStrategyResponse::GetStartNodeConfig(get_start_node_config_response) => {
                get_start_node_config_response
                    .error
                    .as_ref()
                    .unwrap()
                    .clone()
            }
            BacktestStrategyResponse::NodeReset(node_reset_response) => {
                node_reset_response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestStrategyResponse::GetStartNodeConfig(get_start_node_config_response) => {
                get_start_node_config_response.datetime
            }
            BacktestStrategyResponse::NodeReset(node_reset_response) => {
                node_reset_response.datetime
            }
        }
    }
}

#[derive(Debug)]
pub struct GetStartNodeConfigResponse {
    pub success: bool,
    pub node_id: NodeId,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub backtest_strategy_config: BacktestStrategyConfig,
    pub datetime: DateTimeUtc,
}

impl GetStartNodeConfigResponse {
    pub fn success(node_id: NodeId, backtest_strategy_config: BacktestStrategyConfig) -> Self {
        Self {
            success: true,
            node_id,
            error: None,
            backtest_strategy_config,
            datetime: Utc::now()
        }
    }
}

impl From<GetStartNodeConfigResponse> for StrategyResponse {
    fn from(response: GetStartNodeConfigResponse) -> Self {
        StrategyResponse::BacktestStrategy(BacktestStrategyResponse::GetStartNodeConfig(response))
    }
}

#[derive(Debug)]
pub struct NodeResetResponse {
    pub node_id: NodeId,
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl NodeResetResponse {
    pub fn success(node_id: NodeId) -> Self {
        Self {
            node_id,
            success: true,
            error: None,
            datetime: Utc::now()
        }
    }
}

impl From<NodeResetResponse> for StrategyResponse {
    fn from(response: NodeResetResponse) -> Self {
        StrategyResponse::BacktestStrategy(BacktestStrategyResponse::NodeReset(response))
    }
}

#[derive(Debug)]
pub enum BacktestNodeResponse {
    GetStrategyCacheKeys(GetStrategyCacheKeysResponse),
    GetMinIntervalSymbols(GetMinIntervalSymbolsResponse),
    GetCurrentTime(GetCurrentTimeResponse),
}

impl NodeResponseTrait for BacktestNodeResponse {
    fn success(&self) -> bool {
        match self {
            BacktestNodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.success
            }
            BacktestNodeResponse::GetMinIntervalSymbols(get_min_interval_symbols_response) => {
                get_min_interval_symbols_response.success
            }
            BacktestNodeResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.success
            }
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            BacktestNodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response
                    .error
                    .as_ref()
                    .unwrap()
                    .clone()
            }
            BacktestNodeResponse::GetMinIntervalSymbols(get_min_interval_symbols_response) => {
                get_min_interval_symbols_response.error.as_ref().unwrap().clone()
            }
            BacktestNodeResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestNodeResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.datetime
            }
            BacktestNodeResponse::GetMinIntervalSymbols(get_min_interval_symbols_response) => {
                get_min_interval_symbols_response.datetime
            }
            BacktestNodeResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.datetime
            }
        }
    }
}

#[derive(Debug)]
pub struct GetStrategyCacheKeysResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub keys: Vec<Key>,
    pub datetime: DateTimeUtc,
}

impl GetStrategyCacheKeysResponse {
    pub fn success(keys: Vec<Key>) -> Self {
        Self {
            success: true,
            error: None,
            keys,
            datetime: Utc::now()
        }
    }
}

impl From<GetStrategyCacheKeysResponse> for NodeResponse {
    fn from(response: GetStrategyCacheKeysResponse) -> Self {
        NodeResponse::BacktestNode(BacktestNodeResponse::GetStrategyCacheKeys(response))
    }
}


#[derive(Debug)]
pub struct GetMinIntervalSymbolsResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub keys: Vec<Key>,
    pub datetime: DateTimeUtc,
}

impl GetMinIntervalSymbolsResponse {
    pub fn success(keys: Vec<Key>) -> Self {
        Self {
            success: true,
            error: None,
            keys,
            datetime: Utc::now()
        }
    }
}

impl From<GetMinIntervalSymbolsResponse> for NodeResponse {
    fn from(response: GetMinIntervalSymbolsResponse) -> Self {
        NodeResponse::BacktestNode(BacktestNodeResponse::GetMinIntervalSymbols(response))
    }
}



#[derive(Debug)]
pub struct GetCurrentTimeResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub current_time: DateTime<Utc>,
    pub datetime: DateTimeUtc,
}

impl GetCurrentTimeResponse {
    pub fn success(current_time: DateTime<Utc>) -> Self {
        Self {
            success: true,
            error: None,
            current_time,
            datetime: Utc::now()
        }
    }
}

impl From<GetCurrentTimeResponse> for NodeResponse {
    fn from(response: GetCurrentTimeResponse) -> Self {
        NodeResponse::BacktestNode(BacktestNodeResponse::GetCurrentTime(response))
    }
}
