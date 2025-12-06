use ::entity::position;
use sea_orm::*;
use star_river_core::position::Position;

pub struct PositionQuery;

impl PositionQuery {
    pub async fn get_all_positions_by_strategy_id(db: &DbConn, strategy_id: i32) -> Result<Vec<Position>, DbErr> {
        let position_models = position::Entity::find()
            .filter(position::Column::StrategyId.eq(strategy_id))
            .all(db)
            .await?;

        let mut positions = Vec::new();
        for model in position_models {
            let position = model.into();
            positions.push(position);
        }

        Ok(positions)
    }

    pub async fn get_position_number_by_strategy_id(db: &DbConn, strategy_id: i32) -> Result<u64, DbErr> {
        let result = position::Entity::find()
            .filter(position::Column::StrategyId.eq(strategy_id))
            .count(db)
            .await?;
        Ok(result)
    }
}
