pub mod custom_variable;
pub mod sys_varibale;
pub mod strategy_benchmark;
pub mod node_benchmark;
pub mod log_message;
mod tests;

use crate::custom_type::FeeRate;
use crate::market::Exchange;
use crate::market::deserialize_exchange;
use sys_varibale::SysVariable;
use crate::system::DateTimeUtc;
use chrono::{DateTime, Duration, Utc};
use custom_variable::CustomVariable;
use entity::strategy_config::Model as StrategyConfigModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use strum::{Display, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StrategyConfig {
    /// 策略ID
    pub id: i32,
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 策略状态
    pub status: String,
    /// 是否删除
    pub is_deleted: bool,
    /// 交易模式
    pub trade_mode: TradeMode,
    /// 策略配置
    pub config: Option<serde_json::Value>,
    /// 策略节点
    pub nodes: Option<serde_json::Value>,
    /// 策略边
    pub edges: Option<serde_json::Value>,
    /// 实盘图表配置
    pub live_chart_config: Option<serde_json::Value>,
    /// 回测图表配置
    pub backtest_chart_config: Option<serde_json::Value>,
    /// 创建时间
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub created_time: DateTimeUtc,
    /// 更新时间
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub updated_time: DateTimeUtc,
}

impl From<StrategyConfigModel> for StrategyConfig {
    fn from(model: StrategyConfigModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            status: model.status,
            is_deleted: model.is_deleted,
            trade_mode: TradeMode::from_str(model.trade_mode.as_str()).unwrap(),
            config: model.config,
            nodes: model.nodes,
            edges: model.edges,
            live_chart_config: model.live_chart_config,
            backtest_chart_config: model.backtest_chart_config,
            created_time: model.created_time,
            updated_time: model.updated_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum TradeMode {
    #[strum(serialize = "backtest")]
    Backtest, // 回测
    // #[strum(serialize = "simulated")]
    // Simulated, // 模拟
    #[strum(serialize = "live")]
    Live, // 实盘
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SelectedAccount {
    #[serde(rename = "id")]
    pub account_id: i32, // 账户ID

    #[serde(rename = "accountName")]
    pub account_name: String, // 账户名称

    #[serde(deserialize_with = "deserialize_exchange")]
    pub exchange: Exchange, // 交易所

    #[serde(rename = "availableBalance")]
    pub available_balance: f64, // 可用余额
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    String, // 字符串
    Int,    // 整数
    Float,  // 浮点数
}

// 变量

// 实盘模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStrategyConfig {
    #[serde(rename = "liveAccounts")]
    pub live_accounts: Vec<SelectedAccount>, // 账户ID列表
    #[serde(rename = "variables")]
    pub variables: Option<HashMap<String, CustomVariable>>, // 变量 var_name -> Variable
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum BacktestDataSource {
    File, // 文件
    Exchange, // 交易所
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(rename = "startDate")]
    pub start_date: DateTimeUtc, // 开始日期
    #[serde(rename = "endDate")]
    pub end_date: DateTimeUtc, // 结束日期
}

impl TimeRange {
    pub fn new(start_date_str: String, end_date_str: String) -> Self {
        use chrono::NaiveDateTime;

        // 尝试解析RFC 3339格式（如：1971-01-01T00:00:00Z）
        let start_date = match DateTimeUtc::from_str(&start_date_str) {
            Ok(dt) => dt,
            Err(_) => {
                // 如果RFC 3339格式失败，尝试解析"YYYY-MM-DD HH:MM:SS"格式
                match NaiveDateTime::parse_from_str(&start_date_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_dt) => naive_dt.and_utc(),
                    Err(e) => panic!("Failed to parse start_date '{}': {}", start_date_str, e),
                }
            }
        };

        let end_date = match DateTimeUtc::from_str(&end_date_str) {
            Ok(dt) => dt,
            Err(_) => {
                // 如果RFC 3339格式失败，尝试解析"YYYY-MM-DD HH:MM:SS"格式
                match NaiveDateTime::parse_from_str(&end_date_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_dt) => naive_dt.and_utc(),
                    Err(e) => panic!("Failed to parse end_date '{}': {}", end_date_str, e),
                }
            }
        };

        Self { start_date, end_date }
    }

    pub fn duration(&self) -> Duration {
        self.end_date.signed_duration_since(self.start_date)
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ~ {}", self.start_date, self.end_date)
    }
}

pub fn deserialize_time_range<'de, D>(deserializer: D) -> Result<TimeRange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let time_range_value = serde_json::Value::deserialize(deserializer)?;

    if let serde_json::Value::Object(map) = time_range_value {
        let start_date_str = map.get("startDate").and_then(|v| v.as_str());
        let end_date_str = map.get("endDate").and_then(|v| v.as_str());

        if let (Some(start), Some(end)) = (start_date_str, end_date_str) {
            match (
                //前端返回的2025-09-13 00:00:00 +08:00格式 自带时区，解析为DateTime<Utc>
                DateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S %z"),
                DateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S %z"),
            ) {
                (Ok(start_with_tz), Ok(end_with_tz)) => {
                    // 转换为UTC时区
                    let start_date = start_with_tz.with_timezone(&Utc);
                    let end_date = end_with_tz.with_timezone(&Utc);
                    return Ok(TimeRange { start_date, end_date });
                }
                _ => {
                    return Err(serde::de::Error::custom(
                        "can't parse date format, expected format: YYYY-MM-DD HH:MM:SS +TZ:TZ",
                    ));
                }
            }
        }
    }

    Err(serde::de::Error::custom("date format is incorrect"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceExchange {
    #[serde(rename = "id")]
    pub account_id: i32, // 账户ID
    #[serde(rename = "accountName")]
    pub account_name: String, // 账户名称
    #[serde(deserialize_with = "deserialize_exchange")]
    pub exchange: Exchange, // 交易所
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeModeConfig {
    #[serde(rename = "selectedAccounts")]
    pub selected_accounts: Vec<SelectedAccount>,

    #[serde(rename = "timeRange")]
    #[serde(deserialize_with = "deserialize_time_range")]
    pub time_range: TimeRange,
}

// 回测模式配置
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

impl Default for BacktestStrategyConfig {
    fn default() -> Self {
        BacktestStrategyConfig {
            data_source: BacktestDataSource::File,
            exchange_mode_config: None,
            initial_balance: 10000.0,
            leverage: 10,
            fee_rate: 0.0001,
            play_speed: 1,
            custom_variables: Vec::new(),
        }
    }
}

// 模拟模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedConfig {
    pub simulate_accounts: Vec<i32>,                // 账户ID列表
    pub variables: HashMap<String, CustomVariable>, // 变量 var_name -> Variable
}



#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(untagged)]
pub enum StrategyVariable {
    CustomVariable(CustomVariable),
    SysVariable(SysVariable),
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StrategyConfig {
//     pub live_config: Option<LiveStrategyConfig>, // 实盘交易配置
//     pub backtest_config: Option<BacktestConfig>, // 回测交易配置
//     pub simulated_config: Option<SimulatedConfig>, // 模拟交易配置
// }

// impl Default for StrategyConfig {
//     fn default() -> Self {
//         StrategyConfig {
//             live_config: None,
//             backtest_config: None,
//             simulated_config: None,
//         }
//     }
// }
