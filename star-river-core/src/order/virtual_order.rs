use crate::custom_type::*;
use crate::market::Exchange;
use crate::order::{FuturesOrderSide, OrderStatus, OrderType, TpslType};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// 虚拟订单
pub struct VirtualOrder {
    #[serde(rename = "orderId")]
    pub order_id: OrderId, // 订单ID

    #[serde(rename = "positionId")]
    pub position_id: Option<PositionId>, // 仓位ID

    #[serde(rename = "strategyId")]
    pub strategy_id: StrategyId, // 策略ID

    #[serde(rename = "nodeId")]
    pub node_id: NodeId, // 节点ID

    #[serde(rename = "orderConfigId")]
    pub order_config_id: i32, // 订单配置ID

    #[serde(rename = "exchange")]
    pub exchange: Exchange, // 交易所

    #[serde(rename = "symbol")]
    pub symbol: String, // 交易对

    #[serde(rename = "orderSide")]
    pub order_side: FuturesOrderSide, // 订单方向

    #[serde(rename = "orderType")]
    pub order_type: OrderType, // 订单类型

    #[serde(rename = "orderStatus")]
    pub order_status: OrderStatus, // 订单状态

    #[serde(rename = "quantity")]
    pub quantity: f64, // 数量

    #[serde(rename = "openPrice")]
    pub open_price: f64, // 开仓价格

    #[serde(rename = "tp")]
    pub tp: Option<f64>, // 止盈

    #[serde(rename = "sl")]
    pub sl: Option<f64>, // 止损

    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>, // 创建时间

    #[serde(rename = "updateTime")]
    pub update_time: DateTime<Utc>, // 更新时间
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
            tp: Self::calculate_tp(open_price, tp, &tp_type, &order_side),
            sl: Self::calculate_sl(open_price, sl, &sl_type, &order_side),
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
    ) -> Option<f64> {
        // 止盈
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
    ) -> Option<f64> {
        // 止损
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
                }
            }
            return None;
        }
        None
    }
}
