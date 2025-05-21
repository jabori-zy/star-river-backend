use super::SysVariableFunction;
use database::mutation::strategy_sys_variable_mutation::StrategySysVariableMutation;
use database::query::position_query::PositionQuery;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use types::strategy::sys_varibale::StrategySysVariable;

// 更新持仓数量
impl SysVariableFunction {
    pub async fn update_position_number(db: &DatabaseConnection, strategy_id : i32) -> Result<StrategySysVariable, DbErr> {
        // 1. 从仓位表中获取持仓数量
        let position_number = PositionQuery::get_position_number_by_strategy_id(db, strategy_id).await;
        if let Ok(position_number) = position_number {
            // 2. 更新系统变量
            let sys_variable = StrategySysVariableMutation::update_position_number(db, strategy_id, position_number as i32).await;
            // 3. 返回系统变量
            sys_variable
        } else {
            Err(DbErr::Custom("获取持仓数量失败".to_string()))
        }
    }
}
