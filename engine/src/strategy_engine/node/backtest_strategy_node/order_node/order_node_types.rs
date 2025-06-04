use types::order::{OrderType, OrderSide};
use serde::{Serialize, Deserialize};
use types::order::{deserialize_order_type, deserialize_order_side};
use types::strategy::{BacktestDataSource, DataSourceExchange, TimeRange};

// 订单配置
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





#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderNodeBacktestConfig {
    #[serde(rename = "dataSource")]
    pub data_source: BacktestDataSource,
    #[serde(rename = "orderConfig")]
    pub order_config: OrderConfig,
    #[serde(rename = "exchangeConfig")]
    pub exchange_config: Option<OrderNodeExchangeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderNodeExchangeConfig {
    #[serde(rename = "selectedDataSource")]
    pub selected_data_source: DataSourceExchange,
    pub symbol: String,
    #[serde(rename = "timeRange")]
    pub time_range: TimeRange,
}



