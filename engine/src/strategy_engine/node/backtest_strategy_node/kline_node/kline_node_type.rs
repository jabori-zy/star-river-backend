use serde::{Deserialize, Serialize};
use types::market::KlineInterval;
use types::strategy::{BacktestDataSource, SelectedAccount};
use types::strategy::TimeRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,
    #[serde(rename = "fileConfig")]
    pub file_config: Option<FileConfig>,
    #[serde(rename = "exchangeModeConfig")]
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
    pub time_range: TimeRange,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedSymbol {
    #[serde(rename="configId")]
    pub config_id: i32,

    #[serde(rename="outputHandleId")]
    pub output_handle_id: String,

    #[serde(rename="symbol")]
    pub symbol: String,

    #[serde(rename="interval")]
    pub interval: KlineInterval,
}
