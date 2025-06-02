use serde::{Deserialize, Serialize};
use types::market::KlineInterval;
use types::strategy::{BacktestDataSource, SelectedAccount, DataSourceExchange};
use std::str::FromStr;
use types::strategy::TimeRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,
    #[serde(rename = "fileConfig")]
    pub file_config: Option<FileConfig>,
    #[serde(rename = "exchangeConfig")]
    pub exchange_config: Option<ExchangeConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    #[serde(rename = "filePath")]
    pub file_path: String,
}
 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    #[serde(rename = "selectedDataSource")]
    pub selected_data_source: DataSourceExchange,
    pub symbol: String,
    pub interval: KlineInterval,
    #[serde(rename = "timeRange")]
    pub time_range: TimeRange,
}
