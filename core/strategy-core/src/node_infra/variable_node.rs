pub mod trigger;
pub mod variable_config;
pub mod variable_operation;

#[cfg(test)]
mod tests;

// use crate::strategy::backtest_strategy::BacktestDataSource;
use serde::Deserialize;
// Re-export VariableConfig and its subtypes
pub use variable_config::VariableConfig;

use crate::strategy::SelectedAccount;

#[derive(Debug, Clone, Deserialize)]
pub struct VariableNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: Option<String>,
    pub variables: Vec<VariableConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetVariableNodeSimulateConfig {
    pub variables: Vec<VariableConfig>,
}

// Backtest configuration

// Exchange mode configuration
#[derive(Debug, Clone, Deserialize)]
pub struct VariableNodeExchangeModeConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account: SelectedAccount,
}

// #[derive(Debug, Clone, Deserialize)]
// pub struct VariableNodeBacktestConfig {
//     #[serde(rename = "dataSource")]
//     pub data_source: BacktestDataSource,

//     #[serde(rename = "exchangeModeConfig")]
//     pub exchange_mode_config: Option<VariableNodeExchangeModeConfig>,

//     #[serde(rename = "variableConfigs")]
//     pub variable_configs: Vec<VariableConfig>,
// }
