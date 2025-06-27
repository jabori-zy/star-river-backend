use serde::{Deserialize, Serialize};
use crate::strategy::sys_varibale::SysVariable;
use crate::strategy::SelectedAccount;
use crate::strategy::{BacktestDataSource,DataSourceExchange,TimeRange};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeUnit {
    #[serde(rename = "second")]
    Second,
    #[serde(rename = "minute")]
    Minute,
    #[serde(rename = "hour")]
    Hour,
    #[serde(rename = "day")]
    Day,
}

// 获取变量的方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GetVariableType {
    #[serde(rename = "condition")]
    Condition, // 条件触发
    #[serde(rename = "timer")]
    Timer, // 定时触发
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    pub unit: TimeUnit,
    pub interval: u32,
}

impl TimerConfig {
    pub fn get_millisecond(&self) -> u64 {
        match self.unit {
            TimeUnit::Second => self.interval as u64 * 1000,
            TimeUnit::Minute => self.interval as u64 * 60 * 1000,
            TimeUnit::Hour => self.interval as u64 * 60 * 60 * 1000,
            TimeUnit::Day => self.interval as u64 * 24 * 60 * 60 * 1000,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableConfig {
    #[serde(rename = "configId")]
    pub config_id: i32,

    #[serde(rename = "inputHandleId")]
    pub input_handle_id: String,

    #[serde(rename = "symbol")]
    pub symbol: Option<String>,

    #[serde(rename = "getVariableType")]
    pub get_variable_type: GetVariableType,

    #[serde(rename = "timerConfig")]
    pub timer_config: Option<TimerConfig>,

    #[serde(rename = "variableName")]
    pub variable_name: String, // 变量名称

    pub variable: SysVariable, // 变量类型，使用StrategySysVariable的值
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: Option<String>,
    pub variables: Vec<VariableConfig>,
    #[serde(rename = "getVariableType")]
    pub get_variable_type: GetVariableType,
    #[serde(rename = "timerConfig")]
    pub timer_config: Option<TimerConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeSimulateConfig {
    pub variables: Vec<VariableConfig>,
}



// 回测配置


//交易所模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNodeExchangeModeConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account : SelectedAccount,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<VariableNodeExchangeModeConfig>,

    #[serde(rename = "variableConfigs")]
    pub variable_configs: Vec<VariableConfig>,
}