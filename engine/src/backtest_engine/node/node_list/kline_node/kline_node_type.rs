use serde::{Deserialize, Serialize};
use star_river_core::market::KlineInterval;
use strategy_core::strategy::TimeRange;
use star_river_core::strategy::deserialize_time_range;
use star_river_core::strategy::{BacktestDataSource, SelectedAccount};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineNodeBacktestConfig {
    pub data_source: BacktestDataSource,
    pub file_config: Option<FileConfig>,
    pub exchange_mode_config: Option<KlineNodeExchangeModeConfig>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedSymbol {
    #[serde(rename = "configId")]
    pub config_id: i32,

    #[serde(rename = "outputHandleId")]
    pub output_handle_id: String,

    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "interval")]
    pub interval: KlineInterval,
}
