// External crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// Current crate imports
use star_river_core::{
    custom_type::FeeRate,
    system::{TimeRange, deserialize_time_range},
};
use strategy_core::{strategy::SelectedAccount, variable::custom_variable::CustomVariable};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum BacktestDataSource {
    File,     // File source
    Exchange, // Exchange source
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestStrategyConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource, // Data source

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<ExchangeModeConfig>, // Exchange mode configuration

    #[serde(rename = "initialBalance")]
    pub initial_balance: f64, // Initial balance

    #[serde(rename = "leverage")]
    pub leverage: i32, // Leverage

    #[serde(rename = "feeRate")]
    pub fee_rate: FeeRate, // Fee rate

    #[serde(rename = "playSpeed")]
    pub play_speed: i32, // Playback speed

    #[serde(rename = "customVariables")]
    pub custom_variables: Vec<CustomVariable>, // Variables: var_name -> Variable
}

impl BacktestStrategyConfig {
    pub fn start_time(&self) -> Option<DateTime<Utc>> {
        self.exchange_mode_config.as_ref().map(|config| config.time_range.start_date)
    }

    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        self.exchange_mode_config.as_ref().map(|config| config.time_range.end_date)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeModeConfig {
    #[serde(rename = "selectedAccounts")]
    pub selected_accounts: Vec<SelectedAccount>,

    #[serde(rename = "timeRange")]
    #[serde(deserialize_with = "deserialize_time_range")]
    pub time_range: TimeRange,
}
