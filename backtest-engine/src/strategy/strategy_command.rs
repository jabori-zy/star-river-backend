// External crate imports
use derive_more::From;
// Current crate imports
use key::{IndicatorKey, Key, KlineKey};
use star_river_core::{
    custom_type::NodeId,
    kline::{Kline, KlineInterval},
};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    communication::{
        StrategyCommandTrait,
        strategy::{StrategyCommand, StrategyResponse},
    },
    node_infra::variable_node::variable_config::UpdateVariableConfig,
    variable::{custom_variable::CustomVariable, sys_varibale::SysVariable},
};
use ta_lib::Indicator;

#[derive(Debug, From)]
pub enum BacktestStrategyCommand {
    GetStrategyKeys(GetStrategyKeysCommand),
    GetMinInterval(GetMinIntervalCommand),
    InitKlineData(InitKlineDataCommand),
    AppendKlineData(AppendKlineDataCommand),
    InitIndicatorData(InitIndicatorDataCommand),
    GetKlineData(GetKlineDataCommand),
    GetIndicatorData(GetIndicatorDataCommand),
    UpdateKlineData(UpdateKlineDataCommand),
    UpdateIndicatorData(UpdateIndicatorDataCommand),
    InitCustomVariableValue(InitCustomVarValueCommand),
    GetCustomVariableValue(GetCustomVarValueCommand),
    UpdateCustomVariableValue(UpdateCustomVarValueCommand),
    ResetCustomVariableValue(ResetCustomVarValueCommand),
    UpdateSysVariableValue(UpdateSysVarValueCommand),
    AddNodeCycleTracker(AddNodeCycleTrackerCommand),
}

impl BacktestStrategyCommand {
    pub fn node_id(&self) -> NodeId {
        match self {
            BacktestStrategyCommand::GetStrategyKeys(command) => command.node_id(),
            BacktestStrategyCommand::GetMinInterval(command) => command.node_id(),
            BacktestStrategyCommand::InitKlineData(command) => command.node_id(),
            BacktestStrategyCommand::AppendKlineData(command) => command.node_id(),
            BacktestStrategyCommand::InitIndicatorData(command) => command.node_id(),
            BacktestStrategyCommand::GetKlineData(command) => command.node_id(),
            BacktestStrategyCommand::GetIndicatorData(command) => command.node_id(),
            BacktestStrategyCommand::UpdateKlineData(command) => command.node_id(),
            BacktestStrategyCommand::UpdateIndicatorData(command) => command.node_id(),
            BacktestStrategyCommand::InitCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::GetCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::UpdateCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::ResetCustomVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::UpdateSysVariableValue(command) => command.node_id(),
            BacktestStrategyCommand::AddNodeCycleTracker(command) => command.node_id(),
        }
    }
}

impl StrategyCommandTrait for BacktestStrategyCommand {}

// define type aliases
// get strategy keys
pub type GetStrategyKeysCommand = StrategyCommand<GetStrategyKeysCmdPayload, GetStrategyKeysRespPayload>;
pub type GetStrategyKeysResponse = StrategyResponse<GetStrategyKeysRespPayload>;
// get min interval
pub type GetMinIntervalCommand = StrategyCommand<GetMinIntervalCmdPayload, GetMinIntervalRespPayload>;
pub type GetMinIntervalResponse = StrategyResponse<GetMinIntervalRespPayload>;
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
pub type InitCustomVarValueCommand = StrategyCommand<InitCustomVarCmdPayload, InitCustomVarRespPayload>;
pub type InitCustomVarValueResponse = StrategyResponse<InitCustomVarRespPayload>;

// get custom variable value
pub type GetCustomVarValueCommand = StrategyCommand<GetCustomVarCmdPayload, GetCustomVarRespPayload>;
pub type GetCustomVarValueResponse = StrategyResponse<GetCustomVarRespPayload>;
// update custom variable value
pub type UpdateCustomVarValueCommand = StrategyCommand<UpdateCustomVarValueCmdPayload, UpdateCustomVarRespPayload>;
pub type UpdateCustomVarValueResponse = StrategyResponse<UpdateCustomVarRespPayload>;
// reset custom variable value
pub type ResetCustomVarValueCommand = StrategyCommand<ResetCustomVarCmdPayload, ResetCustomVarRespPayload>;
pub type ResetCustomVarValueResponse = StrategyResponse<ResetCustomVarRespPayload>;

// update sys variable value
pub type UpdateSysVarValueCommand = StrategyCommand<UpdateSysVarCmdPayload, UpdateSysVarRespPayload>;
pub type UpdateSysVarValueResponse = StrategyResponse<UpdateSysVarRespPayload>;

// add node cycle tracker
pub type AddNodeCycleTrackerCommand = StrategyCommand<AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerRespPayload>;
pub type AddNodeCycleTrackerResponse = StrategyResponse<AddNodeCycleTrackerRespPayload>;

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
pub struct GetMinIntervalCmdPayload;

#[derive(Debug)]
pub struct GetMinIntervalRespPayload {
    pub interval: KlineInterval,
}

impl GetMinIntervalRespPayload {
    pub fn new(interval: KlineInterval) -> Self {
        Self { interval }
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
    pub play_index: Option<i32>,
    pub limit: Option<i32>,
}

impl GetKlineDataCmdPayload {
    pub fn new(kline_key: KlineKey, play_index: Option<i32>, limit: Option<i32>) -> Self {
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
    pub play_index: Option<i32>,
    pub limit: Option<i32>,
}

impl GetIndicatorDataCmdPayload {
    pub fn new(indicator_key: IndicatorKey, play_index: Option<i32>, limit: Option<i32>) -> Self {
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
pub struct InitCustomVarCmdPayload {
    pub custom_variables: Vec<CustomVariable>,
}

impl InitCustomVarCmdPayload {
    pub fn new(custom_variables: Vec<CustomVariable>) -> Self {
        Self { custom_variables }
    }
}

#[derive(Debug)]
pub struct InitCustomVarRespPayload;

// ============ Get Custom Variable Value ============
#[derive(Debug)]
pub struct GetCustomVarCmdPayload {
    pub var_name: String,
}

impl GetCustomVarCmdPayload {
    pub fn new(var_name: String) -> Self {
        Self { var_name }
    }
}

#[derive(Debug)]
pub struct GetCustomVarRespPayload {
    pub custom_variable: CustomVariable,
}

impl GetCustomVarRespPayload {
    pub fn new(custom_variable: CustomVariable) -> Self {
        Self { custom_variable }
    }
}

// ============ Update Custom Variable Value ============
#[derive(Debug)]
pub struct UpdateCustomVarValueCmdPayload {
    pub update_var_config: UpdateVariableConfig,
}

impl UpdateCustomVarValueCmdPayload {
    pub fn new(update_var_config: UpdateVariableConfig) -> Self {
        Self { update_var_config }
    }
}

#[derive(Debug)]
pub struct UpdateCustomVarRespPayload {
    pub custom_variable: CustomVariable,
}

impl UpdateCustomVarRespPayload {
    pub fn new(custom_variable: CustomVariable) -> Self {
        Self { custom_variable }
    }
}

// ============ Reset Custom Variable Value ============
#[derive(Debug)]
pub struct ResetCustomVarCmdPayload {
    pub var_name: String,
}

impl ResetCustomVarCmdPayload {
    pub fn new(var_name: String) -> Self {
        Self { var_name }
    }
}

#[derive(Debug)]
pub struct ResetCustomVarRespPayload {
    pub custom_variable: CustomVariable,
}

impl ResetCustomVarRespPayload {
    pub fn new(custom_variable: CustomVariable) -> Self {
        Self { custom_variable }
    }
}

// ============ Update Sys Variable Value ============
#[derive(Debug)]
pub struct UpdateSysVarCmdPayload {
    pub sys_variable: SysVariable,
}

impl UpdateSysVarCmdPayload {
    pub fn new(sys_variable: SysVariable) -> Self {
        Self { sys_variable }
    }
}

#[derive(Debug)]
pub struct UpdateSysVarRespPayload;

// ============ Add Node Cycle Tracker ============
#[derive(Debug)]
pub struct AddNodeCycleTrackerCmdPayload {
    pub node_id: NodeId,
    pub cycle_tracker: CompletedCycle,
}

impl AddNodeCycleTrackerCmdPayload {
    pub fn new(node_id: NodeId, cycle_tracker: CompletedCycle) -> Self {
        Self { node_id, cycle_tracker }
    }
}

#[derive(Debug)]
pub struct AddNodeCycleTrackerRespPayload;
