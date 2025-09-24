use super::super::{NodeResponse, NodeResponseTrait, StrategyResponse, StrategyResponseTrait};
use chrono::{DateTime, Utc};
use star_river_core::cache::Key;
use star_river_core::cache::key::KlineKey;
use star_river_core::custom_type::NodeId;
use star_river_core::error::error_trait::StarRiverErrorTrait;
use star_river_core::indicator::Indicator;
use star_river_core::market::Kline;
use star_river_core::strategy::BacktestStrategyConfig;
use star_river_core::system::DateTimeUtc;
use std::sync::Arc;

#[derive(Debug)]
pub enum BacktestNodeResponse {
    GetStartNodeConfig(GetStartNodeConfigResponse),
    NodeReset(NodeResetResponse),
}

impl StrategyResponseTrait for BacktestNodeResponse {
    fn success(&self) -> bool {
        match self {
            BacktestNodeResponse::GetStartNodeConfig(get_start_node_config_response) => {
                get_start_node_config_response.success
            }
            BacktestNodeResponse::NodeReset(node_reset_response) => node_reset_response.success,
        }
    }

    fn node_id(&self) -> &NodeId {
        match self {
            BacktestNodeResponse::GetStartNodeConfig(get_start_node_config_response) => {
                &get_start_node_config_response.node_id
            }
            BacktestNodeResponse::NodeReset(node_reset_response) => &node_reset_response.node_id,
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            BacktestNodeResponse::GetStartNodeConfig(get_start_node_config_response) => {
                get_start_node_config_response.error.as_ref().unwrap().clone()
            }
            BacktestNodeResponse::NodeReset(node_reset_response) => {
                node_reset_response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestNodeResponse::GetStartNodeConfig(get_start_node_config_response) => {
                get_start_node_config_response.datetime
            }
            BacktestNodeResponse::NodeReset(node_reset_response) => node_reset_response.datetime,
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
            datetime: Utc::now(),
        }
    }
}

impl From<GetStartNodeConfigResponse> for StrategyResponse {
    fn from(response: GetStartNodeConfigResponse) -> Self {
        StrategyResponse::BacktestStrategy(BacktestNodeResponse::GetStartNodeConfig(response))
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
            datetime: Utc::now(),
        }
    }
}

impl From<NodeResetResponse> for StrategyResponse {
    fn from(response: NodeResetResponse) -> Self {
        StrategyResponse::BacktestStrategy(BacktestNodeResponse::NodeReset(response))
    }
}

















#[derive(Debug)]
pub enum BacktestStrategyResponse {
    GetStrategyCacheKeys(GetStrategyCacheKeysResponse),
    GetMinIntervalSymbols(GetMinIntervalSymbolsResponse),
    GetCurrentTime(GetCurrentTimeResponse),
    InittKlineData(InitKlineDataResponse),
    InitIndicatorData(InitIndicatorDataResponse),
    GetKlineData(GetKlineDataResponse),
    GetIndicatorData(GetIndicatorDataResponse),
    UpdateKlineData(UpdateKlineDataResponse),
    UpdateIndicatorData(UpdateIndicatorDataResponse),
}

impl NodeResponseTrait for BacktestStrategyResponse {
    fn success(&self) -> bool {
        match self {
            BacktestStrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.success
            }
            BacktestStrategyResponse::GetMinIntervalSymbols(get_min_interval_symbols_response) => {
                get_min_interval_symbols_response.success
            }
            BacktestStrategyResponse::GetCurrentTime(get_current_time_response) => get_current_time_response.success,
            BacktestStrategyResponse::InittKlineData(init_backtest_kline_data_response) => {
                init_backtest_kline_data_response.success
            }
            BacktestStrategyResponse::InitIndicatorData(init_backtest_indicator_data_response) => {
                init_backtest_indicator_data_response.success
            }
            BacktestStrategyResponse::GetKlineData(get_kline_data_response) => {
                get_kline_data_response.success
            }
            BacktestStrategyResponse::GetIndicatorData(get_indicator_data_response) => {
                get_indicator_data_response.success
            }
            BacktestStrategyResponse::UpdateKlineData(update_kline_data_response) => {
                update_kline_data_response.success
            }
            BacktestStrategyResponse::UpdateIndicatorData(update_indicator_data_response) => {
                update_indicator_data_response.success
            }
        }
    }

    fn error(&self) -> Arc<dyn StarRiverErrorTrait> {
        match self {
            BacktestStrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::GetMinIntervalSymbols(get_min_interval_symbols_response) => {
                get_min_interval_symbols_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::GetCurrentTime(get_current_time_response) => {
                get_current_time_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::InittKlineData(init_backtest_kline_data_response) => {
                init_backtest_kline_data_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::InitIndicatorData(init_backtest_indicator_data_response) => {
                init_backtest_indicator_data_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::GetKlineData(get_kline_data_response) => {
                get_kline_data_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::GetIndicatorData(get_indicator_data_response) => {
                get_indicator_data_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::UpdateKlineData(update_kline_data_response) => {
                update_kline_data_response.error.as_ref().unwrap().clone()
            }
            BacktestStrategyResponse::UpdateIndicatorData(update_indicator_data_response) => {
                update_indicator_data_response.error.as_ref().unwrap().clone()
            }
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestStrategyResponse::GetStrategyCacheKeys(get_strategy_cache_keys_response) => {
                get_strategy_cache_keys_response.datetime
            }
            BacktestStrategyResponse::GetMinIntervalSymbols(get_min_interval_symbols_response) => {
                get_min_interval_symbols_response.datetime
            }
            BacktestStrategyResponse::GetCurrentTime(get_current_time_response) => get_current_time_response.datetime,
            BacktestStrategyResponse::InittKlineData(init_backtest_kline_data_response) => {
                init_backtest_kline_data_response.datetime
            }
            BacktestStrategyResponse::InitIndicatorData(init_backtest_indicator_data_response) => {
                init_backtest_indicator_data_response.datetime
            }
            BacktestStrategyResponse::GetKlineData(get_kline_data_response) => {
                get_kline_data_response.datetime
            }
            BacktestStrategyResponse::GetIndicatorData(get_indicator_data_response) => {
                get_indicator_data_response.datetime
            }
            BacktestStrategyResponse::UpdateKlineData(update_kline_data_response) => {
                update_kline_data_response.datetime
            }
            BacktestStrategyResponse::UpdateIndicatorData(update_indicator_data_response) => {
                update_indicator_data_response.datetime
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
            datetime: Utc::now(),
        }
    }
}

impl From<GetStrategyCacheKeysResponse> for NodeResponse {
    fn from(response: GetStrategyCacheKeysResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::GetStrategyCacheKeys(response))
    }
}

#[derive(Debug)]
pub struct GetMinIntervalSymbolsResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub keys: Vec<KlineKey>,
    pub datetime: DateTimeUtc,
}

impl GetMinIntervalSymbolsResponse {
    pub fn success(keys: Vec<KlineKey>) -> Self {
        Self {
            success: true,
            error: None,
            keys,
            datetime: Utc::now(),
        }
    }
}

impl From<GetMinIntervalSymbolsResponse> for NodeResponse {
    fn from(response: GetMinIntervalSymbolsResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::GetMinIntervalSymbols(response))
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
            datetime: Utc::now(),
        }
    }
}

impl From<GetCurrentTimeResponse> for NodeResponse {
    fn from(response: GetCurrentTimeResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::GetCurrentTime(response))
    }
}



#[derive(Debug)]

pub struct InitKlineDataResponse {
    pub node_id: NodeId,
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}

impl InitKlineDataResponse {
    pub fn success(node_id: NodeId) -> Self {
        Self {
            node_id,
            success: true,
            error: None,
            datetime: Utc::now(),
        }
    }
}

impl From<InitKlineDataResponse> for NodeResponse {
    fn from(response: InitKlineDataResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::InittKlineData(response))
    }
}



#[derive(Debug)]
pub struct InitIndicatorDataResponse {
    pub node_id: NodeId,
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub datetime: DateTimeUtc,
}


impl InitIndicatorDataResponse {
    pub fn success(node_id: NodeId) -> Self {
        Self {
            node_id,
            success: true,
            error: None,
            datetime: Utc::now(),
        }
    }
}

impl From<InitIndicatorDataResponse> for NodeResponse {
    fn from(response: InitIndicatorDataResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::InitIndicatorData(response))
    }
}


#[derive(Debug)]
pub struct GetKlineDataResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub data: Vec<Kline>,
    pub datetime: DateTimeUtc,
}


impl GetKlineDataResponse {
    pub fn success(data: Vec<Kline>) -> Self {
        Self {
            success: true,
            error: None,
            data,
            datetime: Utc::now(),
        }
    }
}

impl From<GetKlineDataResponse> for NodeResponse {
    fn from(response: GetKlineDataResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::GetKlineData(response))
    }
}


#[derive(Debug)]
pub struct GetIndicatorDataResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub data: Vec<Indicator>,
    pub datetime: DateTimeUtc,
}

impl GetIndicatorDataResponse {
    pub fn success(data: Vec<Indicator>) -> Self {
        Self {
            success: true,
            error: None,
            data,
            datetime: Utc::now(),
        }
    }

}

impl From<GetIndicatorDataResponse> for NodeResponse {
    fn from(response: GetIndicatorDataResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::GetIndicatorData(response))
    }
}

#[derive(Debug)]
pub struct UpdateKlineDataResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub data: Kline,
    pub datetime: DateTimeUtc,
}


impl UpdateKlineDataResponse {
    pub fn success(data: Kline) -> Self {
        Self {
            success: true,
            error: None,
            data,
            datetime: Utc::now(),
        }
    }
}

impl From<UpdateKlineDataResponse> for NodeResponse {
    fn from(response: UpdateKlineDataResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::UpdateKlineData(response))
    
    }
}

#[derive(Debug)]
pub struct UpdateIndicatorDataResponse {
    pub success: bool,
    pub error: Option<Arc<dyn StarRiverErrorTrait>>,
    pub data: Indicator,
    pub datetime: DateTimeUtc,
}


impl UpdateIndicatorDataResponse {
    pub fn success(data: Indicator) -> Self {
        Self {
            success: true,
            error: None,
            data,
            datetime: Utc::now(),
        }
    }
}

impl From<UpdateIndicatorDataResponse> for NodeResponse {
    fn from(response: UpdateIndicatorDataResponse) -> Self {
        NodeResponse::BacktestNode(BacktestStrategyResponse::UpdateIndicatorData(response))
    }
}