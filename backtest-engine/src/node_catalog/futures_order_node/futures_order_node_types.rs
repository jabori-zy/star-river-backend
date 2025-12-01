use serde::Deserialize;
use snafu::OptionExt;
use star_river_core::{
    custom_type::{InputHandleId, NodeName},
    order::{FuturesOrderSide, OrderType, TpslType},
    system::{TimeRange, deserialize_time_range},
};
use strategy_core::{node_infra::condition_trigger::ConditionTrigger, strategy::SelectedAccount};

use crate::{
    node::node_error::{
        FuturesOrderNodeError,
        futures_order_node_error::{ExchangeModeNotConfiguredSnafu, OrderConfigNotFoundSnafu},
    },
    strategy::strategy_config::BacktestDataSource,
};

// 合约订单配置
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrderConfig {
    pub order_config_id: i32,

    pub input_handle_id: InputHandleId,

    pub symbol: String,

    pub order_type: OrderType,

    // #[serde(deserialize_with = "deserialize_futures_order_side")]
    pub order_side: FuturesOrderSide,

    pub price: f64,

    pub quantity: f64,

    pub tp: Option<f64>,
    pub sl: Option<f64>,

    pub tp_type: Option<TpslType>,

    pub sl_type: Option<TpslType>,

    pub trigger_config: ConditionTrigger,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrderNodeConfig {
    #[serde(skip)]
    pub node_name: NodeName,

    pub data_source: BacktestDataSource,

    pub exchange_mode_config: Option<FuturesOrderNodeExchangeModeConfig>,

    pub futures_order_configs: Vec<FuturesOrderConfig>,
}

impl FuturesOrderNodeConfig {
    pub fn exchange_mode(&self) -> Result<&FuturesOrderNodeExchangeModeConfig, FuturesOrderNodeError> {
        if let Some(exchange_mode_config) = &self.exchange_mode_config {
            Ok(exchange_mode_config)
        } else {
            Err(ExchangeModeNotConfiguredSnafu {
                node_name: self.node_name.clone(),
            }
            .build())
        }
    }

    pub fn find_order_config(&self, config_id: i32) -> Result<&FuturesOrderConfig, FuturesOrderNodeError> {
        self.futures_order_configs
            .iter()
            .find(|config| config.order_config_id == config_id)
            .context(OrderConfigNotFoundSnafu {
                order_config_id: config_id,
            })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrderNodeExchangeModeConfig {
    pub selected_account: SelectedAccount,
    #[serde(deserialize_with = "deserialize_time_range")]
    pub time_range: TimeRange,
}
