pub mod variable_operation;
pub mod variable_config;
pub mod trigger;

#[cfg(test)]
mod tests;

use crate::strategy::SelectedAccount;
use crate::strategy::BacktestDataSource;
use serde::{Deserialize, Serialize};

// 重新导出 VariableConfig 及其子类型
pub use variable_config::VariableConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: Option<String>,
    pub variables: Vec<VariableConfig>,
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
    pub selected_account: SelectedAccount,
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
