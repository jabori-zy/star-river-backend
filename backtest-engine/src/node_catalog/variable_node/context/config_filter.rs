use strategy_core::node_infra::variable_node::{
    VariableConfig,
    trigger::{ConditionTrigger, TriggerConfig},
};

/// Filter variable configurations matching case condition trigger
///
/// # Arguments
/// * `variable_configs` - Iterator of variable configurations
/// * `case_id` - Case identifier
/// * `from_node_id` - Source node ID
///
/// # Returns
/// Filtered vector of variable configurations
pub fn filter_case_trigger_configs<'a, I>(variable_configs: I, case_id: i32, from_node_id: &String) -> Vec<VariableConfig>
where
    I: Iterator<Item = &'a VariableConfig>,
{
    variable_configs
        .filter(|config| matches!(config.trigger_config(), TriggerConfig::Condition(_)))
        .filter(|config| match config.trigger_config() {
            TriggerConfig::Condition(condition_trigger) => match condition_trigger {
                ConditionTrigger::Case(case_trigger) => case_trigger.case_id == case_id && &case_trigger.from_node_id == from_node_id,
                ConditionTrigger::Else(_) => false,
            },
            _ => false,
        })
        .cloned()
        .collect()
}

/// Filter variable configurations matching else trigger
///
/// # Arguments
/// * `variable_configs` - Iterator of variable configurations
/// * `from_node_id` - Source node ID
///
/// # Returns
/// Filtered vector of variable configurations
pub fn filter_else_trigger_configs<'a, I>(variable_configs: I, from_node_id: &String) -> Vec<VariableConfig>
where
    I: Iterator<Item = &'a VariableConfig>,
{
    variable_configs
        .filter(|config| matches!(config.trigger_config(), TriggerConfig::Condition(_)))
        .filter(|config| match config.trigger_config() {
            TriggerConfig::Condition(condition_trigger) => match condition_trigger {
                ConditionTrigger::Case(_) => false,
                ConditionTrigger::Else(else_trigger) => &else_trigger.from_node_id == from_node_id,
            },
            _ => false,
        })
        .cloned()
        .collect()
}

/// Filter variable configurations matching dataflow trigger
///
/// # Arguments
/// * `variable_configs` - Iterator of variable configurations
/// * `from_node_id` - Source node ID
/// * `config_id` - Configuration ID
///
/// # Returns
/// Filtered vector of variable configurations
pub fn filter_dataflow_trigger_configs<'a, I>(variable_configs: I, from_node_id: &String, config_id: i32) -> Vec<VariableConfig>
where
    I: Iterator<Item = &'a VariableConfig>,
{
    variable_configs
        .filter(|config| matches!(config.trigger_config(), TriggerConfig::Dataflow(_)))
        .filter(|config| match config.trigger_config() {
            TriggerConfig::Dataflow(dataflow_trigger) => {
                dataflow_trigger.from_var_config_id == config_id && &dataflow_trigger.from_node_id == from_node_id
            }
            _ => false,
        })
        .cloned()
        .collect()
}
