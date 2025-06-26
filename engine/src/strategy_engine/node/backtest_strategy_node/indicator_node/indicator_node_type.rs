
use types::indicator::IndicatorConfig;
use serde::{Deserialize, Serialize};
use types::strategy::TimeRange;
use types::strategy::{BacktestDataSource, SelectedAccount};
use super::super::kline_node::kline_node_type::SelectedSymbol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeConfig")]
    pub exchange_mode_config: Option<ExchangeModeConfig>,
    
    #[serde(rename = "fileConfig")]
    pub file_mode_config: Option<FileModeConfig>,
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedIndicator {
    #[serde(rename="indicatorId")]
    pub indicator_id: i32,

    #[serde(rename="handleId")]
    pub handle_id: String,

    #[serde(rename = "indicatorConfig")]
    pub indicator_config: IndicatorConfig,
}



