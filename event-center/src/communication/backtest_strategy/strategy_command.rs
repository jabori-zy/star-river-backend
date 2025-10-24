use super::{StrategyCommand, StrategyResponse};
use derive_more::From;
use star_river_core::custom_type::PlayIndex;
use star_river_core::indicator::Indicator;
use star_river_core::key::Key;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::market::Kline;
use star_river_core::node::variable_node::variable_config::UpdateVariableConfig;
use star_river_core::strategy::custom_variable::{CustomVariable, VariableValue};
use star_river_core::strategy::sys_varibale::SysVariable;
use star_river_core::system::DateTimeUtc;

// define type aliases
// get strategy keys
pub type GetStrategyKeysCommand = StrategyCommand<GetStrategyKeysCmdPayload, GetStrategyKeysRespPayload>;
pub type GetStrategyKeysResponse = StrategyResponse<GetStrategyKeysRespPayload>;
// get min interval symbols
pub type GetMinIntervalSymbolsCommand = StrategyCommand<GetMinIntervalSymbolsCmdPayload, GetMinIntervalSymbolsRespPayload>;
pub type GetMinIntervalSymbolsResponse = StrategyResponse<GetMinIntervalSymbolsRespPayload>;
// get current time
pub type GetCurrentTimeCommand = StrategyCommand<GetCurrentTimeCmdPayload, GetCurrentTimeRespPayload>;
pub type GetCurrentTimeResponse = StrategyResponse<GetCurrentTimeRespPayload>;
// init kline data
pub type InitKlineDataCommand = StrategyCommand<InitKlineDataCmdPayload, InitKlineDataRespPayload>;
pub type InitKlineDataResponse = StrategyResponse<InitKlineDataRespPayload>;
// init indicator data
pub type InitIndicatorDataCommand = StrategyCommand<InitIndicatorDataCmdPayload, InitIndicatorDataRespPayload>;
pub type InitIndicatorDataResponse = StrategyResponse<InitIndicatorDataRespPayload>;
// append kline data
pub type AppendKlineDataCommand = StrategyCommand<AppendKlineDataCmdPayload, AppendKlineDataRespPayload>;
pub type AppendKlineDataResponse = StrategyResponse<AppendKlineDataRespPayload>;
// get kline data
pub type GetKlineDataCommand = StrategyCommand<GetKlineDataCmdPayload, GetKlineDataRespPayload>;
pub type GetKlineDataResponse = StrategyResponse<GetKlineDataRespPayload>;
// get indicator data
pub type GetIndicatorDataCommand = StrategyCommand<GetIndicatorDataCmdPayload, GetIndicatorDataRespPayload>;
pub type GetIndicatorDataResponse = StrategyResponse<GetIndicatorDataRespPayload>;
// update kline data
pub type UpdateKlineDataCommand = StrategyCommand<UpdateKlineDataCmdPayload, UpdateKlineDataRespPayload>;
pub type UpdateKlineDataResponse = StrategyResponse<UpdateKlineDataRespPayload>;
// update indicator data
pub type UpdateIndicatorDataCommand = StrategyCommand<UpdateIndicatorDataCmdPayload, UpdateIndicatorDataRespPayload>;
pub type UpdateIndicatorDataResponse = StrategyResponse<UpdateIndicatorDataRespPayload>;
// init custom variable value
pub type InitCustomVariableValueCommand = StrategyCommand<InitCustomVariableCmdPayload, InitCustomVariableRespPayload>;
pub type InitCustomVariableValueResponse = StrategyResponse<InitCustomVariableRespPayload>;

// get custom variable value
pub type GetCustomVariableValueCommand = StrategyCommand<GetCustomVariableCmdPayload, GetCustomVariableRespPayload>;
pub type GetCustomVariableValueResponse = StrategyResponse<GetCustomVariableRespPayload>;
// update custom variable value
pub type UpdateCustomVariableValueCommand = StrategyCommand<UpdateCustomVariableValueCmdPayload, UpdateCustomVariableRespPayload>;
pub type UpdateCustomVariableValueResponse = StrategyResponse<UpdateCustomVariableRespPayload>;
// reset custom variable value
pub type ResetCustomVariableValueCommand = StrategyCommand<ResetCustomVariableCmdPayload, ResetCustomVariableRespPayload>;
pub type ResetCustomVariableValueResponse = StrategyResponse<ResetCustomVariableRespPayload>;

// update sys variable value
pub type UpdateSysVariableValueCommand = StrategyCommand<UpdateSysVariableCmdPayload, UpdateSysVariableRespPayload>;
pub type UpdateSysVariableValueResponse = StrategyResponse<UpdateSysVariableRespPayload>;

// ============ Get Strategy Keys ============
#[derive(Debug, From)]
pub struct GetStrategyKeysCmdPayload;

#[derive(Debug)]
pub struct GetStrategyKeysRespPayload {
    pub keys: Vec<Key>,
}

impl GetStrategyKeysRespPayload {
    pub fn new(keys: Vec<Key>) -> Self {
        Self { keys }
    }
}

// ============ Get Min Interval Symbols ============
#[derive(Debug)]
pub struct GetMinIntervalSymbolsCmdPayload;

#[derive(Debug)]
pub struct GetMinIntervalSymbolsRespPayload {
    pub keys: Vec<KlineKey>,
}

impl GetMinIntervalSymbolsRespPayload {
    pub fn new(keys: Vec<KlineKey>) -> Self {
        Self { keys }
    }
}

// ============ Get Current Time ============
#[derive(Debug)]
pub struct GetCurrentTimeCmdPayload;

#[derive(Debug)]
pub struct GetCurrentTimeRespPayload {
    pub current_time: DateTimeUtc,
}

impl GetCurrentTimeRespPayload {
    pub fn new(current_time: DateTimeUtc) -> Self {
        Self { current_time }
    }
}

// ============ Init Kline Data ============
#[derive(Debug)]
pub struct InitKlineDataCmdPayload {
    pub kline_key: KlineKey,
    pub init_kline_data: Vec<Kline>,
}

impl InitKlineDataCmdPayload {
    pub fn new(kline_key: KlineKey, init_kline_data: Vec<Kline>) -> Self {
        Self {
            kline_key,
            init_kline_data,
        }
    }
}

#[derive(Debug)]
pub struct InitKlineDataRespPayload;

// ============ Init Indicator Data ============
#[derive(Debug)]
pub struct InitIndicatorDataCmdPayload {
    pub indicator_key: IndicatorKey,
    pub indicator_series: Vec<Indicator>,
}

impl InitIndicatorDataCmdPayload {
    pub fn new(indicator_key: IndicatorKey, indicator_series: Vec<Indicator>) -> Self {
        Self {
            indicator_key,
            indicator_series,
        }
    }
}

#[derive(Debug)]
pub struct InitIndicatorDataRespPayload;

// ============ Append Kline Data ============
#[derive(Debug)]
pub struct AppendKlineDataCmdPayload {
    pub kline_key: KlineKey,
    pub kline_series: Vec<Kline>,
}

impl AppendKlineDataCmdPayload {
    pub fn new(kline_key: KlineKey, kline_series: Vec<Kline>) -> Self {
        Self { kline_key, kline_series }
    }
}

#[derive(Debug)]
pub struct AppendKlineDataRespPayload;

// ============ Get Kline Data ============
#[derive(Debug)]
pub struct GetKlineDataCmdPayload {
    pub kline_key: KlineKey,
    pub play_index: Option<PlayIndex>,
    pub limit: Option<i32>,
}

impl GetKlineDataCmdPayload {
    pub fn new(kline_key: KlineKey, play_index: Option<PlayIndex>, limit: Option<i32>) -> Self {
        Self {
            kline_key,
            play_index,
            limit,
        }
    }
}

#[derive(Debug)]
pub struct GetKlineDataRespPayload {
    pub kline_series: Vec<Kline>,
}

impl GetKlineDataRespPayload {
    pub fn new(kline_series: Vec<Kline>) -> Self {
        Self { kline_series }
    }
}

// ============ Get Indicator Data ============
#[derive(Debug)]
pub struct GetIndicatorDataCmdPayload {
    pub indicator_key: IndicatorKey,
    pub play_index: Option<PlayIndex>,
    pub limit: Option<i32>,
}

impl GetIndicatorDataCmdPayload {
    pub fn new(indicator_key: IndicatorKey, play_index: Option<PlayIndex>, limit: Option<i32>) -> Self {
        Self {
            indicator_key,
            play_index,
            limit,
        }
    }
}

#[derive(Debug)]
pub struct GetIndicatorDataRespPayload {
    pub indicator_series: Vec<Indicator>,
}

impl GetIndicatorDataRespPayload {
    pub fn new(data: Vec<Indicator>) -> Self {
        Self { indicator_series: data }
    }
}

// ============ Update Kline Data ============
#[derive(Debug)]
pub struct UpdateKlineDataCmdPayload {
    pub kline_key: KlineKey,
    pub kline: Kline,
}

impl UpdateKlineDataCmdPayload {
    pub fn new(kline_key: KlineKey, kline: Kline) -> Self {
        Self { kline_key, kline }
    }
}

#[derive(Debug)]
pub struct UpdateKlineDataRespPayload {
    pub data: Kline,
}

impl UpdateKlineDataRespPayload {
    pub fn new(data: Kline) -> Self {
        Self { data }
    }
}

// ============ Update Indicator Data ============
#[derive(Debug)]
pub struct UpdateIndicatorDataCmdPayload {
    pub indicator_key: IndicatorKey,
    pub indicator: Indicator,
}

impl UpdateIndicatorDataCmdPayload {
    pub fn new(indicator_key: IndicatorKey, indicator: Indicator) -> Self {
        Self { indicator_key, indicator }
    }
}

#[derive(Debug)]
pub struct UpdateIndicatorDataRespPayload {
    pub data: Indicator,
}

impl UpdateIndicatorDataRespPayload {
    pub fn new(data: Indicator) -> Self {
        Self { data }
    }
}



// ============ Init Custom Variable Value ============
#[derive(Debug)]
pub struct InitCustomVariableCmdPayload {
    pub custom_variables: Vec<CustomVariable>,
}



impl InitCustomVariableCmdPayload {
    pub fn new(custom_variables: Vec<CustomVariable>) -> Self {
        Self { custom_variables }
    }
}


#[derive(Debug)]
pub struct InitCustomVariableRespPayload;


// ============ Get Custom Variable Value ============
#[derive(Debug)]
pub struct GetCustomVariableCmdPayload {
    pub var_name: String,
}

impl GetCustomVariableCmdPayload {
    pub fn new(var_name: String) -> Self {
        Self { var_name }
    }
}

#[derive(Debug)]
pub struct GetCustomVariableRespPayload {
    pub custom_variable: CustomVariable,
}

impl GetCustomVariableRespPayload {
    pub fn new(custom_variable: CustomVariable) -> Self {
        Self { custom_variable }
    }
}


// ============ Update Custom Variable Value ============
#[derive(Debug)]
pub struct UpdateCustomVariableValueCmdPayload {
    pub update_var_config: UpdateVariableConfig,
}

impl UpdateCustomVariableValueCmdPayload {
    pub fn new(update_var_config: UpdateVariableConfig) -> Self {
        Self { update_var_config }
    }
}

#[derive(Debug)]
pub struct UpdateCustomVariableRespPayload {
    pub custom_variable: CustomVariable,
}


impl UpdateCustomVariableRespPayload {
    pub fn new(custom_variable: CustomVariable) -> Self {
        Self { custom_variable }
    }
}



// ============ Reset Custom Variable Value ============
#[derive(Debug)]
pub struct ResetCustomVariableCmdPayload {
    pub var_name: String,
}

impl ResetCustomVariableCmdPayload {
    pub fn new(var_name: String) -> Self {
        Self { var_name }
    }
}

#[derive(Debug)]
pub struct ResetCustomVariableRespPayload {
    pub custom_variable: CustomVariable,
}

impl ResetCustomVariableRespPayload {
    pub fn new(custom_variable: CustomVariable) -> Self {
        Self { custom_variable }
    }
}


// ============ Update Sys Variable Value ============
#[derive(Debug)]
pub struct UpdateSysVariableCmdPayload {
    pub sys_variable: SysVariable,
}

impl UpdateSysVariableCmdPayload {
    pub fn new(sys_variable: SysVariable) -> Self {
        Self { sys_variable }
    }
}

#[derive(Debug)]
pub struct UpdateSysVariableRespPayload;