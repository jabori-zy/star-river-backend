// pub mod backtest_strategy;
pub mod context_trait;
pub mod leaf_node_execution_tracker;
pub mod metadata;
pub mod state_machine;
pub mod strategy_trait;
pub mod cycle;

use std::str::FromStr;

use entity::strategy_config::Model as StrategyConfigModel;
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::{StrategyId, StrategyName},
    exchange::{Exchange, deserialize_exchange},
    kline::KlineInterval,
    system::DateTimeUtc,
};
use strum::{Display, EnumString};
use ta_lib::IndicatorConfig;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyConfig {
    /// 策略ID
    pub id: StrategyId,
    /// 策略名称
    pub name: StrategyName,
    /// 策略描述
    pub description: String,
    /// 策略状态
    pub status: String,
    /// 是否删除
    pub is_deleted: bool,
    /// 交易模式
    pub trade_mode: TradeMode,
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
    pub create_time: DateTimeUtc,
    /// 更新时间
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub update_time: DateTimeUtc,
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
            nodes: model.nodes,
            edges: model.edges,
            live_chart_config: model.live_chart_config,
            backtest_chart_config: model.backtest_chart_config,
            create_time: model.create_time,
            update_time: model.update_time,
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
#[serde(rename_all = "camelCase")]
pub struct SelectedSymbol {
    pub config_id: i32,
    pub output_handle_id: String,
    pub symbol: String,
    pub interval: KlineInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedIndicator {
    #[serde(rename = "configId")]
    pub config_id: i32,

    #[serde(rename = "outputHandleId")]
    pub output_handle_id: String,

    #[serde(rename = "indicatorConfig")]
    pub indicator_config: IndicatorConfig,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TimeRange {
//     #[serde(rename = "startDate")]
//     pub start_date: DateTime<Utc>, // 开始日期
//     #[serde(rename = "endDate")]
//     pub end_date: DateTime<Utc>, // 结束日期
// }

// impl TimeRange {
//     pub fn new(start_date_str: String, end_date_str: String) -> Self {
//         use chrono::NaiveDateTime;

//         // 尝试解析RFC 3339格式（如：1971-01-01T00:00:00Z）
//         let start_date = match DateTimeUtc::from_str(&start_date_str) {
//             Ok(dt) => dt,
//             Err(_) => {
//                 // 如果RFC 3339格式失败，尝试解析"YYYY-MM-DD HH:MM:SS"格式
//                 match NaiveDateTime::parse_from_str(&start_date_str, "%Y-%m-%d %H:%M:%S") {
//                     Ok(naive_dt) => naive_dt.and_utc(),
//                     Err(e) => panic!("Failed to parse start_date '{}': {}", start_date_str, e),
//                 }
//             }
//         };

//         let end_date = match DateTimeUtc::from_str(&end_date_str) {
//             Ok(dt) => dt,
//             Err(_) => {
//                 // 如果RFC 3339格式失败，尝试解析"YYYY-MM-DD HH:MM:SS"格式
//                 match NaiveDateTime::parse_from_str(&end_date_str, "%Y-%m-%d %H:%M:%S") {
//                     Ok(naive_dt) => naive_dt.and_utc(),
//                     Err(e) => panic!("Failed to parse end_date '{}': {}", end_date_str, e),
//                 }
//             }
//         };

//         Self { start_date, end_date }
//     }

//     pub fn duration(&self) -> Duration {
//         self.end_date.signed_duration_since(self.start_date)
//     }
// }

// impl fmt::Display for TimeRange {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{} ~ {}", self.start_date, self.end_date)
//     }
// }

// pub fn deserialize_time_range<'de, D>(deserializer: D) -> Result<TimeRange, D::Error>
// where
//     D: serde::Deserializer<'de>,
// {
//     let time_range_value = serde_json::Value::deserialize(deserializer)?;

//     if let serde_json::Value::Object(map) = time_range_value {
//         let start_date_str = map.get("startDate").and_then(|v| v.as_str());
//         let end_date_str = map.get("endDate").and_then(|v| v.as_str());

//         if let (Some(start), Some(end)) = (start_date_str, end_date_str) {
//             match (
//                 //前端返回的2025-09-13 00:00:00 +08:00格式 自带时区，解析为DateTime<Utc>
//                 DateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S %z"),
//                 DateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S %z"),
//             ) {
//                 (Ok(start_with_tz), Ok(end_with_tz)) => {
//                     // 转换为UTC时区
//                     let start_date = start_with_tz.with_timezone(&Utc);
//                     let end_date = end_with_tz.with_timezone(&Utc);
//                     return Ok(TimeRange { start_date, end_date });
//                 }
//                 _ => {
//                     return Err(serde::de::Error::custom(
//                         "can't parse date format, expected format: YYYY-MM-DD HH:MM:SS +TZ:TZ",
//                     ));
//                 }
//             }
//         }
//     }

//     Err(serde::de::Error::custom("date format is incorrect"))
// }
