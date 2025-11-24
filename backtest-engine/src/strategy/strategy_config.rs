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
    File,     // 文件
    Exchange, // 交易所
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestStrategyConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource, // 数据源

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<ExchangeModeConfig>, // 交易所模式配置

    #[serde(rename = "initialBalance")]
    pub initial_balance: f64, // 初始资金

    #[serde(rename = "leverage")]
    pub leverage: i32, // 杠杆

    #[serde(rename = "feeRate")]
    pub fee_rate: FeeRate, // 手续费率

    #[serde(rename = "playSpeed")]
    pub play_speed: i32, // 回放速度

    #[serde(rename = "customVariables")]
    pub custom_variables: Vec<CustomVariable>, // 变量 var_name -> Variable
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
