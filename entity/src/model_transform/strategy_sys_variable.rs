use types::strategy::sys_varibale::StrategySysVariable;
use crate::strategy_sys_variable::Model as StrategySysVariableModel;

impl From<StrategySysVariableModel> for StrategySysVariable {
    fn from(config: StrategySysVariableModel) -> Self {
        StrategySysVariable {
            id: config.id,
            strategy_id: config.strategy_id,
            position_number: config.position_number,
            create_time: config.create_time,
            update_time: config.update_time,
        }
    }
}