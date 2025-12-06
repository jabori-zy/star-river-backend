use strategy_core::node_infra::condition_trigger::ConditionTrigger;

use crate::node_catalog::position_node::position_node_types::PositionOperationConfig;

/// Filter position operation configurations matching Case condition trigger
///
/// # Arguments
/// * `operation_configs` - Iterator of position operation configurations
/// * `case_id` - Case ID
/// * `from_node_id` - Source node ID
///
/// # Returns
/// Vector of filtered operation config IDs
pub fn filter_case_trigger_configs<'a, I>(operation_configs: I, case_id: i32, from_node_id: &String) -> Vec<i32>
where
    I: Iterator<Item = &'a PositionOperationConfig>,
{
    operation_configs
        .filter(|config| match config.trigger_config {
            ConditionTrigger::Case(ref case_trigger) => case_trigger.case_id == case_id && &case_trigger.from_node_id == from_node_id,
            ConditionTrigger::Else(_) => false,
        })
        .map(|config| config.config_id)
        .collect()
}

/// Filter position operation configurations matching Else condition trigger
///
/// # Arguments
/// * `operation_configs` - Iterator of position operation configurations
/// * `from_node_id` - Source node ID
///
/// # Returns
/// Vector of filtered operation config IDs
pub fn filter_else_trigger_configs<'a, I>(operation_configs: I, from_node_id: &String) -> Vec<i32>
where
    I: Iterator<Item = &'a PositionOperationConfig>,
{
    operation_configs
        .filter(|config| match config.trigger_config {
            ConditionTrigger::Case(_) => false,
            ConditionTrigger::Else(ref else_trigger) => &else_trigger.from_node_id == from_node_id,
        })
        .map(|config| config.config_id)
        .collect()
}
