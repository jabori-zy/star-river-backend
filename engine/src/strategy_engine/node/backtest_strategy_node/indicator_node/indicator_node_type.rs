
use types::indicator::IndicatorConfig;
use types::market::{Exchange, KlineInterval};
use serde::{Deserialize, Serialize};
use types::strategy::TimeRange;
use types::strategy::{BacktestDataSource, SelectedAccount, DataSourceExchange};
use types::market::deserialize_exchange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,
    #[serde(rename = "exchangeConfig")]
    pub exchange_config: Option<ExchangeConfig>,
    #[serde(rename = "fileConfig")]
    pub file_config: Option<FileConfig>,
    #[serde(rename = "indicatorConfig")]
    pub indicator_config: IndicatorConfig,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    #[serde(deserialize_with = "deserialize_exchange")]
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    #[serde(rename = "timeRange")]
    pub time_range: TimeRange,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub file_path: String,
}




