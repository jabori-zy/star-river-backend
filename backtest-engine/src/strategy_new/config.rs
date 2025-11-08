use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use strategy_core::variable::custom_variable::CustomVariable;
use star_river_core::custom_type::FeeRate;
use strategy_core::strategy::SelectedAccount;
use strategy_core::strategy::TimeRange;
use strategy_core::strategy::deserialize_time_range;





#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum BacktestDataSource {
    File, // 文件
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




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeModeConfig {
    #[serde(rename = "selectedAccounts")]
    pub selected_accounts: Vec<SelectedAccount>,

    #[serde(rename = "timeRange")]
    #[serde(deserialize_with = "deserialize_time_range")]
    pub time_range: TimeRange,
}