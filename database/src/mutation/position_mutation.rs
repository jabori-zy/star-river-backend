use ::entity::position;
use sea_orm::*;
use star_river_core::{
    exchange::Exchange,
    position::{OriginalPosition, Position, PositionState},
};

pub struct PositionMutation;

impl PositionMutation {
    pub async fn insert_position(
        db: &DbConn,
        strategy_id: i64,
        node_id: String,
        account_id: i32,
        exchange_position: Box<dyn OriginalPosition>,
    ) -> Result<Position, DbErr> {
        match exchange_position.get_exchange() {
            Exchange::Metatrader5(_) => {
                let extra_info = exchange_position.get_extra_info();
                let position_model = position::ActiveModel {
                    id: NotSet,
                    exchange_position_id: Set(exchange_position.get_exchange_position_id()),
                    node_id: Set(node_id),
                    strategy_id: Set(strategy_id),
                    account_id: Set(account_id),
                    exchange: Set(exchange_position.get_exchange().to_string()),
                    symbol: Set(exchange_position.get_symbol()),
                    position_side: Set(exchange_position.get_position_side().to_string()),
                    position_state: Set(PositionState::Open.to_string()),
                    quantity: Set(exchange_position.get_quantity()),
                    open_price: Set(exchange_position.get_open_price()),
                    unrealized_profit: Set(exchange_position.get_unrealized_profit()),
                    sl: Set(exchange_position.get_sl()),
                    tp: Set(exchange_position.get_tp()),
                    extra_info: Set(extra_info),
                    created_time: Set(exchange_position.get_create_time().to_utc()),
                    updated_time: Set(exchange_position.get_update_time().to_utc()),
                }
                .insert(db)
                .await?;

                Ok(position_model.into()) // 将数据库模型转换系统模型
            }
            _ => {
                return Err(DbErr::Custom("不支持的交易所".to_string()));
            }
        }
    }

    pub async fn update_position(
        db: &DbConn,
        latest_position: Position, // 最新的持仓
    ) -> Result<Position, DbErr> {
        let position_id = latest_position.position_id;
        let position: position::ActiveModel = position::Entity::find_by_id(position_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find position.".to_owned()))
            .map(Into::into)?;

        let position_model = position::ActiveModel {
            id: position.id,
            position_side: Set(latest_position.position_side.to_string()),
            quantity: Set(latest_position.quantity),
            open_price: Set(latest_position.open_price),
            sl: Set(latest_position.sl),
            tp: Set(latest_position.tp),
            extra_info: Set(latest_position.extra_info),
            created_time: Set(latest_position.create_time.to_utc()),
            updated_time: Set(latest_position.update_time.to_utc()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(position_model.into())
    }

    pub async fn update_position_state(db: &DbConn, position_id: i32, position_state: PositionState) -> Result<Position, DbErr> {
        let position: position::ActiveModel = position::Entity::find_by_id(position_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find position.".to_owned()))
            .map(Into::into)?;

        let position_model = position::ActiveModel {
            id: position.id,
            position_state: Set(position_state.to_string()),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(position_model.into())
    }
}
