use types::strategy::SelectedAccount;
use types::strategy::sys_varibale::SysVariable;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableConfig {
    #[serde(rename = "configId")]
    config_id: i32,
    #[serde(rename = "variableName")]
    variable_name: String, // 变量名称
    variable: SysVariable, // 变量类型，使用StrategySysVariable的值
    #[serde(rename = "selectedAccount")]
    selected_account: Vec<SelectedAccount>,
    symbol: Option<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeLiveConfig {
    pub variables: Vec<GetVariableConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeSimulateConfig {
    pub variables: Vec<GetVariableConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetVariableNodeBacktestConfig {
    pub variables: Vec<GetVariableConfig>,
}