use ::entity::strategy_config::Entity as StrategyConfigEntity;
use sea_orm::*;

pub struct StrategySysVariableQuery;

impl StrategySysVariableQuery {
    // Get strategy position number
    pub async fn get_strategy_position_number(db: &DbConn, strategy_id: i32) -> Result<i32, DbErr> {
        // Select only position_number field
        let strategy_sys_variable = StrategyConfigEntity::find_by_id(strategy_id)
            .column(strategy_config::Column::PositionNumber)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))?;

        Ok(strategy_sys_variable.position_number)
    }
}
