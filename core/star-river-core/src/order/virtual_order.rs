use crate::custom_type::*;
use crate::exchange::Exchange;
use crate::order::{FuturesOrderSide, OrderStatus, OrderType, TpslType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// Virtual order
pub struct VirtualOrder {
    #[serde(rename = "orderId")]
    pub order_id: OrderId, // Order ID

    #[serde(rename = "positionId")]
    pub position_id: Option<PositionId>, // Position ID

    #[serde(rename = "strategyId")]
    pub strategy_id: StrategyId, // Strategy ID

    #[serde(rename = "nodeId")]
    pub node_id: NodeId, // Node ID

    #[serde(rename = "orderConfigId")]
    pub order_config_id: i32, // Order config ID

    #[serde(rename = "exchange")]
    pub exchange: Exchange, // Exchange

    #[serde(rename = "symbol")]
    pub symbol: String, // Symbol

    #[serde(rename = "orderSide")]
    pub order_side: FuturesOrderSide, // Order side

    #[serde(rename = "orderType")]
    pub order_type: OrderType, // Order type

    #[serde(rename = "orderStatus")]
    pub order_status: OrderStatus, // Order status

    #[serde(rename = "quantity")]
    pub quantity: f64, // Quantity

    #[serde(rename = "openPrice")]
    pub open_price: f64, // Open price

    #[serde(rename = "tp")]
    pub tp: Option<f64>, // Take profit

    #[serde(rename = "sl")]
    pub sl: Option<f64>, // Stop loss

    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>, // Create time

    #[serde(rename = "updateTime")]
    pub update_time: DateTime<Utc>, // Update time
}

impl VirtualOrder {
    pub fn new(
        position_id: Option<PositionId>,
        order_id: OrderId,
        strategy_id: StrategyId,
        node_id: NodeId,
        order_config_id: i32,
        exchange: Exchange,
        symbol: String,
        order_side: FuturesOrderSide,
        order_type: OrderType,
        quantity: f64,
        open_price: f64,
        tp: Option<f64>,
        sl: Option<f64>,
        tp_type: Option<TpslType>,
        sl_type: Option<TpslType>,
        point: Option<f64>,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            position_id,
            order_id,
            strategy_id,
            node_id,
            order_config_id,
            exchange,
            symbol,
            order_side: order_side.clone(),
            order_type,
            quantity,
            open_price,
            tp: Self::calculate_tp(open_price, tp, &tp_type, &order_side, point),
            sl: Self::calculate_sl(open_price, sl, &sl_type, &order_side, point),
            order_status: OrderStatus::Created,
            create_time: datetime,
            update_time: datetime,
        }
    }

    fn calculate_tp(
        open_price: f64,
        tp: Option<f64>,
        tp_type: &Option<TpslType>,
        order_side: &FuturesOrderSide,
        point: Option<f64>,
    ) -> Option<f64> {
        // Take profit
        if let Some(tp) = tp {
            if let Some(tp_type) = tp_type.clone() {
                match tp_type {
                    TpslType::Price => {
                        return Some(tp);
                    }
                    TpslType::Percentage => match order_side {
                        FuturesOrderSide::OpenLong => {
                            return Some(open_price * (1.0 + tp / 100.0));
                        }
                        FuturesOrderSide::OpenShort => {
                            return Some(open_price * (1.0 - tp / 100.0));
                        }
                        _ => return None,
                    },
                    TpslType::Point => {
                        if let Some(point) = point {
                            match order_side {
                                FuturesOrderSide::OpenLong => {
                                    return Some(open_price + tp * point);
                                }
                                FuturesOrderSide::OpenShort => {
                                    return Some(open_price - tp * point);
                                }
                                _ => return None,
                            }
                        }
                        return None;
                    }
                }
            }
            return None;
        }
        None
    }

    fn calculate_sl(
        open_price: f64,
        sl: Option<f64>,
        sl_type: &Option<TpslType>,
        order_side: &FuturesOrderSide,
        point: Option<f64>,
    ) -> Option<f64> {
        // Stop loss
        if let Some(sl) = sl {
            if let Some(sl_type) = sl_type.clone() {
                match sl_type {
                    TpslType::Price => {
                        return Some(sl);
                    }
                    TpslType::Percentage => match order_side {
                        FuturesOrderSide::OpenLong => {
                            return Some(open_price * (1.0 - sl / 100.0));
                        }
                        FuturesOrderSide::OpenShort => {
                            return Some(open_price * (1.0 + sl / 100.0));
                        }
                        _ => return None,
                    },
                    TpslType::Point => {
                        if let Some(point) = point {
                            match order_side {
                                FuturesOrderSide::OpenLong => {
                                    return Some(open_price - sl * point);
                                }
                                FuturesOrderSide::OpenShort => {
                                    return Some(open_price + sl * point);
                                }
                                _ => return None,
                            }
                        }
                        return None;
                    }
                }
            }
            return None;
        }
        None
    }
}
