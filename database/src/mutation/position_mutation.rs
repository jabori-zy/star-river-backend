
use types::position::ExchangePosition as TypeExchangePosition;
use sea_orm::*;
use crate::entities::position;
use types::position::Position as TypePosition;


pub struct PositionMutation;


impl PositionMutation {
    pub async fn insert_position(
        db: &DbConn,
        strategy_id: i64,
        node_id: String,
        exchange_position: Box<dyn TypeExchangePosition>
    ) -> Result<TypePosition, DbErr> {
        let position_model = position::ActiveModel {
            id: NotSet,
            exchange_position_id: Set(exchange_position.get_exchange_position_id()),
            node_id: Set(node_id),
            strategy_id: Set(strategy_id),
            exchange: Set(exchange_position.get_exchange().to_string()),
            symbol: Set(exchange_position.get_symbol()),
            position_side: Set(exchange_position.get_position_side().to_string()),
            quantity: Set(exchange_position.get_quantity()),
            open_price: Set(exchange_position.get_open_price()),
            sl: Set(exchange_position.get_sl()),
            tp: Set(exchange_position.get_tp()),
            created_time: Set(exchange_position.get_create_time()),
            updated_time: Set(exchange_position.get_update_time()),
            ..Default::default()
        }.insert(db).await.unwrap();

        Ok(position_model.into()) // 将数据库模型转换系统模型

    }
}