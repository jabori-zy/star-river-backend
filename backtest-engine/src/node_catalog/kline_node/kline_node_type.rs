use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::NodeName,
    system::{TimeRange, deserialize_time_range},
};
use strategy_core::strategy::{SelectedAccount, SelectedSymbol};

use crate::{
    node::node_error::kline_node_error::{ExchangeModeNotConfiguredSnafu, KlineNodeError},
    strategy::strategy_config::BacktestDataSource,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineNodeBacktestConfig {
    node_name: NodeName,
    pub data_source: BacktestDataSource,
    pub file_config: Option<FileConfig>,
    pub exchange_mode_config: Option<KlineNodeExchangeModeConfig>,
}

impl KlineNodeBacktestConfig {
    pub fn exchange_mode(&self) -> Result<&KlineNodeExchangeModeConfig, KlineNodeError> {
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
pub struct FileConfig {
    #[serde(rename = "filePath")]
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineNodeExchangeModeConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account: SelectedAccount,

    #[serde(rename = "selectedSymbols")]
    pub selected_symbols: Vec<SelectedSymbol>,

    #[serde(rename = "timeRange")]
    #[serde(deserialize_with = "deserialize_time_range")]
    pub time_range: TimeRange,
}
