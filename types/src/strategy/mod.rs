pub mod node_event;
pub mod info;
pub mod sys_varibale;// 图表消息
pub mod node_command;
pub mod node_response;
pub mod strategy_inner_event;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::market::Exchange;
use strum::{EnumString, Display};
use chrono::{DateTime, Utc};
use std::fmt;
use utoipa::ToSchema;
use crate::market::deserialize_exchange;
use crate::custom_type::FeeRate;


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StrategyConfig {
    /// 策略ID
    pub id: i32,
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 策略状态
    pub status: i32,
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
    pub created_time: DateTime<Utc>,
    /// 更新时间
    pub updated_time: DateTime<Utc>,
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
    Int, // 整数
    Float, // 浮点数
}

// 变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub var_type: VariableType, // 变量类型
    pub var_name: String, // 变量名称
    pub var_display_name: String, // 变量显示名称
    pub var_value: String, // 变量值
}

// 实盘模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStrategyConfig {
    #[serde(rename = "liveAccounts")]
    pub live_accounts: Vec<SelectedAccount>, // 账户ID列表
    #[serde(rename = "variables")]
    pub variables: Option<HashMap<String, Variable>>, // 变量 var_name -> Variable
}


#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
pub enum BacktestDataSource {
    #[strum(serialize = "file")]
    #[serde(rename = "file")]
    File, // 文件

    #[strum(serialize = "exchange")]
    #[serde(rename = "exchange")]
    Exchange, // 交易所
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(rename = "startDate")]
    pub start_date: chrono::NaiveDate, // 开始日期
    #[serde(rename = "endDate")]
    pub end_date: chrono::NaiveDate, // 结束日期
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ~ {}", self.start_date, self.end_date)
    }
}


fn deserialize_time_range<'de, D>(deserializer: D) -> Result<TimeRange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let time_range_value = serde_json::Value::deserialize(deserializer)?;
    
    if let serde_json::Value::Object(map) = time_range_value {
        let start_date_str = map.get("startDate").and_then(|v| v.as_str());
        let end_date_str = map.get("endDate").and_then(|v| v.as_str());
        
        if let (Some(start), Some(end)) = (start_date_str, end_date_str) {
            match (chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d"), 
                   chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")) {
                (Ok(start_date), Ok(end_date)) => {
                    return Ok(TimeRange { start_date, end_date });
                }
                _ => return Err(serde::de::Error::custom("无法解析日期格式"))
            }
        }
    }
    
    Err(serde::de::Error::custom("日期格式不正确"))
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
    
    #[serde(rename = "variables")]
    pub variables: Vec<Variable>, // 变量 var_name -> Variable
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
            variables: Vec::new(),
        }
    }
}


// 模拟模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedConfig {
    pub simulate_accounts: Vec<i32>, // 账户ID列表
    pub variables: HashMap<String, Variable>, // 变量 var_name -> Variable
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



