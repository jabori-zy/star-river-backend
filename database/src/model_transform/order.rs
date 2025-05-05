use types::order::{Order, OrderSide, OrderType, OrderStatus};
use crate::entities::order::Model as OrderModel;
use types::market::Exchange;
use std::str::FromStr;

impl From<OrderModel> for Order {
    fn from(order: OrderModel) -> Self {
        Order {
            order_id: order.id,
            strategy_id: order.strategy_id,
            node_id: order.node_id,
            exchange_order_id: order.exchange_order_id,
            account_id: order.account_id,
            exchange: Exchange::from_str(&order.exchange).unwrap(),
            symbol: order.symbol,
            order_side: OrderSide::from_str(&order.order_side).unwrap(),
            order_type: OrderType::from_str(&order.order_type).unwrap(),
            order_status: OrderStatus::from_str(&order.order_status).unwrap(),
            quantity: order.quantity,
            open_price: order.price,
            sl: order.sl,
            tp: order.tp,
            extra_info: order.extra_info,
            created_time: order.created_time,
            updated_time: order.updated_time,
        }
    }
}





