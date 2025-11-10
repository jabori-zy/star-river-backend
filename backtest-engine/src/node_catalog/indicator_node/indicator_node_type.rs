use strategy_core::strategy::{SelectedSymbol, SelectedAccount, SelectedIndicator};
use serde::{Deserialize, Serialize};
use ta_lib::IndicatorConfig;
use star_river_core::system::TimeRange;
use crate::strategy::strategy_config::BacktestDataSource;

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

