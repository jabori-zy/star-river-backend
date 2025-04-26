pub mod message;
pub mod info;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::market::Exchange;
use std::str::FromStr;
use strum::{EnumString, Display};

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, Eq, PartialEq, Hash)]
pub enum TradeMode {
    #[strum(serialize = "backtest")]
    Backtest, // 回测
    #[strum(serialize = "simulated")]
    Simulated, // 模拟
    #[strum(serialize = "live")]
    Live, // 实盘
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn deserialize_exchange<'de, D>(deserializer: D) -> Result<Exchange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 首先尝试常规的反序列化
    let exchange_str = String::deserialize(deserializer)?;
    Exchange::from_str(&exchange_str).map_err(serde::de::Error::custom)
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
pub struct LiveConfig {
    #[serde(rename = "liveAccounts")]
    pub live_accounts: Vec<SelectedAccount>, // 账户ID列表
    #[serde(rename = "variables")]
    pub variables: Option<HashMap<String, Variable>>, // 变量 var_name -> Variable
}

// 回测模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub start_date: String, // 开始日期
    pub end_date: String, // 结束日期
    pub accounts: Vec<i32>, // 账户ID列表
    pub variables: HashMap<String, Variable>, // 变量 var_name -> Variable
}

// 模拟模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedConfig {
    pub simulate_accounts: Vec<i32>, // 账户ID列表
    pub variables: HashMap<String, Variable>, // 变量 var_name -> Variable
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub live_config: Option<LiveConfig>, // 实盘交易配置
    pub backtest_config: Option<BacktestConfig>, // 回测交易配置
    pub simulated_config: Option<SimulatedConfig>, // 模拟交易配置
}


impl Default for StrategyConfig {
    fn default() -> Self {
        StrategyConfig {
            live_config: None,
            backtest_config: None,  
            simulated_config: None,
        }
    }
}



