use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use star_river_core::{
    custom_type::*,
    exchange::Exchange,
    order::{FuturesOrderSide, OrderStatus, OrderType, TpslType},
};
use utoipa::ToSchema;

use super::id_generator::ORDER_ID_COUNTER;
use crate::error::{VirtualOrderSerializeFailedSnafu, VtsError};
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VirtualOrder {
    pub order_id: OrderId,               // 订单ID
    pub position_id: Option<PositionId>, // 仓位ID
    pub strategy_id: StrategyId,         // 策略ID
    pub node_id: NodeId,                 // 节点ID
    pub node_name: NodeName,             // 节点名称
    pub order_config_id: i32,            // 订单配置ID
    pub exchange: Exchange,              // 交易所
    pub symbol: String,                  // 交易对
    pub order_side: FuturesOrderSide,    // 订单方向
    pub order_type: OrderType,           // 订单类型
    pub order_status: OrderStatus,       // 订单状态
    pub quantity: f64,                   // 数量
    pub open_price: f64,                 // 开仓价格
    pub tp: Option<f64>,                 // 止盈
    pub sl: Option<f64>,                 // 止损
    pub create_time: DateTime<Utc>,      // 创建时间
    pub update_time: DateTime<Utc>,      // 更新时间
}

impl VirtualOrder {
    pub fn to_value(&self) -> Result<serde_json::Value, VtsError> {
        serde_json::to_value(self).context(VirtualOrderSerializeFailedSnafu {
            virtual_order: self.clone(),
        })
    }

    pub fn new(
        position_id: Option<PositionId>,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
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
        let order_id = ORDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            order_id,
            position_id,
            strategy_id,
            node_id,
            node_name,
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

    pub fn create_order(
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
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
        Self::new(
            None,
            strategy_id,
            node_id,
            node_name,
            order_config_id,
            exchange,
            symbol,
            order_side,
            order_type,
            quantity,
            open_price,
            tp,
            sl,
            tp_type,
            sl_type,
            point,
            datetime,
        )
    }

    pub fn create_take_profit_order(
        position_id: Option<PositionId>,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        order_config_id: i32,
        exchange: Exchange,
        symbol: String,
        order_side: FuturesOrderSide,
        quantity: f64,
        open_price: f64,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::new(
            position_id,
            strategy_id,
            node_id,
            node_name,
            order_config_id,
            exchange,
            symbol,
            order_side,
            OrderType::TakeProfitMarket,
            quantity,
            open_price,
            None,
            None,
            None,
            None,
            None,
            datetime,
        )
    }

    pub fn create_stop_loss_order(
        position_id: Option<PositionId>,
        strategy_id: StrategyId,
        node_id: NodeId,
        node_name: NodeName,
        order_config_id: i32,
        exchange: Exchange,
        symbol: String,
        order_side: FuturesOrderSide,
        quantity: f64,
        open_price: f64,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self::new(
            position_id,
            strategy_id,
            node_id,
            node_name,
            order_config_id,
            exchange,
            symbol,
            order_side,
            OrderType::StopMarket,
            quantity,
            open_price,
            None,
            None,
            None,
            None,
            None,
            datetime,
        )
    }

    fn calculate_tp(
        open_price: f64,
        tp: Option<f64>,
        tp_type: &Option<TpslType>,
        order_side: &FuturesOrderSide,
        point: Option<f64>,
    ) -> Option<f64> {
        // 止盈
        if let Some(tp) = tp {
            if let Some(tp_type) = tp_type.clone() {
                match tp_type {
                    TpslType::Price => {
                        return Some(tp);
                    }
                    TpslType::Percentage => match order_side {
                        FuturesOrderSide::Long => {
                            return Some(open_price * (1.0 + tp / 100.0));
                        }
                        FuturesOrderSide::Short => {
                            return Some(open_price * (1.0 - tp / 100.0));
                        }
                    },
                    TpslType::Point => {
                        if let Some(point) = point {
                            match order_side {
                                FuturesOrderSide::Long => {
                                    return Some(open_price + tp * point);
                                }
                                FuturesOrderSide::Short => {
                                    return Some(open_price - tp * point);
                                }
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
        // 止损
        if let Some(sl) = sl {
            if let Some(sl_type) = sl_type.clone() {
                match sl_type {
                    TpslType::Price => {
                        return Some(sl);
                    }
                    TpslType::Percentage => match order_side {
                        FuturesOrderSide::Long => {
                            return Some(open_price * (1.0 - sl / 100.0));
                        }
                        FuturesOrderSide::Short => {
                            return Some(open_price * (1.0 + sl / 100.0));
                        }
                    },
                    TpslType::Point => {
                        if let Some(point) = point {
                            match order_side {
                                FuturesOrderSide::Long => {
                                    return Some(open_price - sl * point);
                                }
                                FuturesOrderSide::Short => {
                                    return Some(open_price + sl * point);
                                }
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
