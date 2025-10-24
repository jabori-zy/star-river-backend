use ::entity::{strategy_sys_variable, strategy_sys_variable::Entity as StrategySysVariableEntity};
use chrono::Utc;
use sea_orm::*;
use star_river_core::strategy::sys_varibale::SysVariable;

pub struct StrategySysVariableMutation;

impl StrategySysVariableMutation {
    pub async fn insert_strategy_sys_variable(db: &DbConn, strategy_id: i32) -> Result<SysVariable, DbErr> {
        let strategy_sys_variable_model = strategy_sys_variable::ActiveModel {
            id: NotSet,
            strategy_id: Set(strategy_id),
            position_number: Set(0),
            create_time: Set(Utc::now()),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(strategy_sys_variable_model.into())
    }

    pub async fn update_position_number(db: &DbConn, strategy_id: i32, position_number: i32) -> Result<SysVariable, DbErr> {
        let strategy: strategy_sys_variable::ActiveModel = StrategySysVariableEntity::find_by_id(strategy_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find strategy.".to_owned()))
            .map(Into::into)?;

        let strategy_sys_variable_model = strategy_sys_variable::ActiveModel {
            id: strategy.id,
            strategy_id: Set(strategy_id),
            position_number: Set(position_number),
            update_time: Set(Utc::now()),
            ..Default::default()
        }
        .update(db)
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        Ok(strategy_sys_variable_model.into())
    }
}
