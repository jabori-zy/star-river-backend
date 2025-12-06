pub mod custom_variable;
pub mod sys_varibale;
pub mod variable_operation;

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(untagged)]
pub enum StrategyVariable {
    CustomVariable(custom_variable::CustomVariable),
    SysVariable(sys_varibale::SysVariable),
}
