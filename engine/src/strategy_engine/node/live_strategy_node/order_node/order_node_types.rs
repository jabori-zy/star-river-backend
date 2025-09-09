use serde::{Deserialize, Serialize};
use types::order::{deserialize_futures_order_side, deserialize_order_type};
use types::order::{FuturesOrderSide, OrderType};
use types::strategy::SelectedAccount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderConfig {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_order_type")]
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    #[serde(deserialize_with = "deserialize_futures_order_side")]
    #[serde(rename = "orderSide")]
    pub order_side: FuturesOrderSide,
    pub quantity: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    #[serde(rename = "orderConfig")]
    pub order_config: OrderConfig,
}
