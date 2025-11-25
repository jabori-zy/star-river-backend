use serde::{Deserialize, Serialize};
use star_river_core::{custom_type::NodeName, system::TimeRange};
use strategy_core::strategy::{SelectedAccount, SelectedIndicator, SelectedSymbol};

use crate::{
    node::node_error::{
        IndicatorNodeError,
        indicator_node_error::{ExchangeModeNotConfiguredSnafu, FileModeNotConfiguredSnafu},
    },
    strategy::strategy_config::BacktestDataSource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeBacktestConfig {
    #[serde(skip)]
    pub node_name: NodeName,

    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeConfig")]
    pub exchange_mode_config: Option<ExchangeModeConfig>,

    #[serde(rename = "fileConfig")]
    pub file_mode_config: Option<FileModeConfig>,
}

impl IndicatorNodeBacktestConfig {
    pub fn exchange_mode(&self) -> Result<&ExchangeModeConfig, IndicatorNodeError> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeModeConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account: SelectedAccount,

    #[serde(rename = "selectedSymbol")]
    pub selected_symbol: SelectedSymbol,

    #[serde(rename = "selectedIndicators")]
    pub selected_indicators: Vec<SelectedIndicator>,

    #[serde(rename = "timeRange")]
    pub time_range: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModeConfig {
    pub file_path: String,
}
