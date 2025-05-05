use crate::entities::position;
use types::position::Position;
use sea_orm::*;

pub struct PositionQuery;

impl PositionQuery {
    pub async fn get_all_positions_by_strategy_id(db: &DbConn, strategy_id: i32) -> Result<Vec<Position>, DbErr> {
        let positions = position::Entity::find().filter(position::Column::StrategyId.eq(strategy_id)).all(db).await?;
        let result: Vec<Position> = positions.into_iter().map(|p| p.into()).collect();
        Ok(result)
    }

    pub async fn get_position_number_by_strategy_id(db: &DbConn, strategy_id: i32) -> Result<u64, DbErr> {
        let result = position::Entity::find()
            .filter(position::Column::StrategyId.eq(strategy_id))
            .count(db)
            .await?;
        Ok(result)
    }
} 