use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::InputHandleId,
    order::{FuturesOrderSide, OrderType, TpslType},
    system::{TimeRange, deserialize_time_range},
};
use strategy_core::strategy::SelectedAccount;

use crate::strategy::strategy_config::BacktestDataSource;

// 合约订单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderConfig {
    #[serde(rename = "orderConfigId")]
    pub order_config_id: i32,

    #[serde(rename = "inputHandleId")]
    pub input_handle_id: InputHandleId,

    pub symbol: String,

    #[serde(rename = "orderType")]
    pub order_type: OrderType,

    // #[serde(deserialize_with = "deserialize_futures_order_side")]
    #[serde(rename = "orderSide")]
    pub order_side: FuturesOrderSide,

    pub price: f64,

    pub quantity: f64,

    pub tp: Option<f64>,
    pub sl: Option<f64>,

    #[serde(rename = "tpType")]
    pub tp_type: Option<TpslType>,

    #[serde(rename = "slType")]
    pub sl_type: Option<TpslType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,

    #[serde(rename = "exchangeModeConfig")]
    pub exchange_mode_config: Option<FuturesOrderNodeExchangeModeConfig>,

    #[serde(rename = "futuresOrderConfigs")]
    pub futures_order_configs: Vec<FuturesOrderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderNodeExchangeModeConfig {
    #[serde(rename = "selectedAccount")]
    pub selected_account: SelectedAccount,
    #[serde(rename = "timeRange")]
    #[serde(deserialize_with = "deserialize_time_range")]
    pub time_range: TimeRange,
}
