use types::order::{OrderType, OrderSide};
use types::strategy::SelectedAccount;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfig {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_order_type")]
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    #[serde(deserialize_with = "deserialize_order_side")]
    #[serde(rename = "orderSide")]
    pub order_side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
}

fn deserialize_order_type<'de, D>(deserializer: D) -> Result<OrderType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;
    
    // 使用as_str()方法获取&str，然后传递给from_str
    match OrderType::from_str(s.as_str()) {
        Ok(order_type) => Ok(order_type),
        Err(e) => Err(serde::de::Error::custom(format!("无法解析OrderType: {}", e)))
    }
}

fn deserialize_order_side<'de, D>(deserializer: D) -> Result<OrderSide, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;
    
    // 使用as_str()方法获取&str，然后传递给from_str
    match OrderSide::from_str(s.as_str()) {
        Ok(order_side) => Ok(order_side),
        Err(e) => Err(serde::de::Error::custom(format!("无法解析OrderSide: {}", e)))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    #[serde(rename = "orderConfig")]
    pub order_config: OrderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderNodeSimulateConfig {
    #[serde(rename = "selectedSimulateAccount")]
    pub selected_simulate_account: SelectedAccount,
    #[serde(rename = "orderConfig")]
    pub order_config: OrderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderNodeBacktestConfig {
    #[serde(rename = "selectedBacktestAccounts")]
    pub selected_backtest_accounts: Vec<i32>,
    #[serde(rename = "orderConfig")]
    pub order_config: OrderConfig,
}




