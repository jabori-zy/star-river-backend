use serde::Deserialize;
use crate::strategy::strategy_config::BacktestDataSource;
use strategy_core::node_infra::variable_node::VariableNodeExchangeModeConfig;
use strategy_core::node_infra::variable_node::VariableConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct VariableNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<VariableNodeExchangeModeConfig>,

    #[serde(rename = "variableConfigs")]
    pub variable_configs: Vec<VariableConfig>,
}