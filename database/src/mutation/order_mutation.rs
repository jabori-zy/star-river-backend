use ::entity::{order, order::Entity as Order};
use sea_orm::*;
use types::order::Order as TypeOrder;
use types::{market::Exchange, order::OriginalOrder};

pub struct OrderMutation;

impl OrderMutation {
    pub async fn insert_order(
        db: &DbConn,
        strategy_id: i64,
        node_id: String,
        account_id: i32,
        original_order: Box<dyn OriginalOrder>,
    ) -> Result<TypeOrder, DbErr> {
        match original_order.get_exchange() {
            Exchange::Metatrader5(_) => {
                let order_model = order::ActiveModel {
                    id: NotSet,
                    exchange_order_id: Set(original_order.get_exchange_order_id()),
                    node_id: Set(node_id),
                    strategy_id: Set(strategy_id),
                    account_id: Set(account_id),
                    exchange: Set(original_order.get_exchange().to_string()),
                    symbol: Set(original_order.get_symbol()),
                    order_side: Set(original_order.get_order_side().to_string()),
                    order_status: Set(original_order.get_order_status().to_string()),
                    order_type: Set(original_order.get_order_type().to_string()),
                    quantity: Set(original_order.get_quantity()),
                    price: Set(original_order.get_open_price()),
                    sl: Set(original_order.get_sl()),
                    tp: Set(original_order.get_tp()),
                    extra_info: Set(original_order.get_extra_info()),
                    created_time: Set(original_order.get_created_time()),
                    updated_time: Set(original_order.get_updated_time()),
                }
                .insert(db)
                .await
                .unwrap();
                Ok(order_model.into())
            }
            _ => Err(DbErr::Custom("Unsupported exchange.".to_owned())),
        }

        // 将数据库模型转换系统模型
    }

    pub async fn update_order(db: &DbConn, latest_order: TypeOrder) -> Result<TypeOrder, DbErr> {
        let order: order::ActiveModel = Order::find_by_id(latest_order.order_id as i32)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find order.".to_owned()))
            .map(Into::into)?;

        let order_model: order::Model = order::ActiveModel {
            id: order.id,
            exchange_order_id: Set(latest_order.exchange_order_id),
            node_id: Set(latest_order.node_id),
            strategy_id: Set(latest_order.strategy_id as i64),
            exchange: Set(latest_order.exchange.to_string()),
            symbol: Set(latest_order.symbol),
            order_side: Set(latest_order.order_side.to_string()),
            order_status: Set(latest_order.order_status.to_string()),
            order_type: Set(latest_order.order_type.to_string()),
            quantity: Set(latest_order.quantity),
            price: Set(latest_order.open_price),
            sl: Set(latest_order.sl),
            tp: Set(latest_order.tp),
            created_time: Set(latest_order.created_time),
            updated_time: Set(latest_order.updated_time),
            ..Default::default()
        }
        .update(db)
        .await
        .unwrap();

        Ok(order_model.into())
    }
}
