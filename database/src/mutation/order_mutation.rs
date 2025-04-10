
use types::order::Order as TypeOrder;
use sea_orm::*;
use crate::entities::{order, order::Entity as Order};
use chrono::Utc;


pub struct OrderMutation;


impl OrderMutation {
    pub async fn create_order(
        db: &DbConn,
        order: TypeOrder
    ) -> Result<order::Model, DbErr> {
        order::ActiveModel {
            id: NotSet,
            order_id: Set(order.order_id as i32),
            strategy_id: Set(order.strategy_id as i32),
            exchange: Set(order.exchange.to_string()),
            symbol: Set(order.symbol),
            order_side: Set(order.order_side.to_string()),
            order_status: Set(order.order_status.to_string()),
            order_type: Set(order.order_type.to_string()),
            quantity: Set(order.quantity as f32),
            price: Set(order.price as f32),
            sl: Set(order.sl.map(|f| f as f32)),
            tp: Set(order.tp.map(|f| f as f32)),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
            ..Default::default()
        }.insert(db).await

    }
}