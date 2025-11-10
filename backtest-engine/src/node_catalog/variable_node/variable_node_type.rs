use serde::Deserialize;
use strategy_core::node_infra::variable_node::{VariableConfig, VariableNodeExchangeModeConfig};

use crate::strategy::strategy_config::BacktestDataSource;

#[derive(Debug, Clone, Deserialize)]
pub struct VariableNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<VariableNodeExchangeModeConfig>,

    #[serde(rename = "variableConfigs")]
    pub variable_configs: Vec<VariableConfig>,
}
