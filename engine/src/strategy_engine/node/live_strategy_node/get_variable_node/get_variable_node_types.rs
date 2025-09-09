use serde::{Deserialize, Serialize};
use types::strategy::sys_varibale::SysVariable;
use types::strategy::SelectedAccount;

// 获取变量的方式
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct GetVariableConfig {
    #[serde(rename = "configId")]
    pub config_id: String,
    #[serde(rename = "variableName")]
    pub variable_name: String, // 变量名称
    pub variable: SysVariable, // 变量类型，使用StrategySysVariable的值
}

// 时间单位
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: Option<String>,
    pub variables: Vec<GetVariableConfig>,
    #[serde(rename = "getVariableType")]
    pub get_variable_type: GetVariableType,
    #[serde(rename = "timerConfig")]
    pub timer_config: Option<TimerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeSimulateConfig {
    pub variables: Vec<GetVariableConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeBacktestConfig {
    pub variables: Vec<GetVariableConfig>,
}
