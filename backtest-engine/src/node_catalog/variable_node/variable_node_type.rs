use serde::Deserialize;
use star_river_core::custom_type::NodeName;
use strategy_core::node_infra::variable_node::{VariableConfig, VariableNodeExchangeModeConfig};
use crate::{
    node::node_error::{VariableNodeError, variable_node_error::ExchangeModeNotConfiguredSnafu},
    strategy::strategy_config::BacktestDataSource,
};

#[derive(Debug, Clone, Deserialize)]
pub struct VariableNodeBacktestConfig {
    #[serde(skip)]
    pub node_name: NodeName,

    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<VariableNodeExchangeModeConfig>,

    #[serde(rename = "variableConfigs")]
    pub variable_configs: Vec<VariableConfig>,
}

impl VariableNodeBacktestConfig {
    pub fn exchange_mode(&self) -> Result<&VariableNodeExchangeModeConfig, VariableNodeError> {
        if let Some(exchange_mode_config) = &self.exchange_mode_config {
            Ok(exchange_mode_config)
        } else {
            Err(ExchangeModeNotConfiguredSnafu {
                node_name: self.node_name.clone(),
            }
            .build())
        }
    }
}
