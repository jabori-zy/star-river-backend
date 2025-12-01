use strategy_core::node_infra::condition_trigger::ConditionTrigger;

use crate::node_catalog::futures_order_node::futures_order_node_types::FuturesOrderConfig;

/// Filter order configurations matching Case condition trigger
///
/// # Arguments
/// * `order_configs` - Iterator of order configurations
/// * `case_id` - Case ID
/// * `from_node_id` - Source node ID
///
/// # Returns
/// Vector of filtered order config IDs
pub fn filter_case_trigger_configs<'a, I>(order_configs: I, case_id: i32, from_node_id: &String) -> Vec<i32>
where
    I: Iterator<Item = &'a FuturesOrderConfig>,
{
    order_configs
        .filter(|config| match config.trigger_config {
            ConditionTrigger::Case(ref case_trigger) => case_trigger.case_id == case_id && &case_trigger.from_node_id == from_node_id,
            ConditionTrigger::Else(_) => false,
        })
        .map(|config| config.order_config_id)
        .collect()
}

/// Filter order configurations matching Else condition trigger
///
/// # Arguments
/// * `order_configs` - Iterator of order configurations
/// * `from_node_id` - Source node ID
///
/// # Returns
/// Vector of filtered order config IDs
pub fn filter_else_trigger_configs<'a, I>(order_configs: I, from_node_id: &String) -> Vec<i32>
where
    I: Iterator<Item = &'a FuturesOrderConfig>,
{
    order_configs
        .filter(|config| match config.trigger_config {
            ConditionTrigger::Case(_) => false,
            ConditionTrigger::Else(ref else_trigger) => &else_trigger.from_node_id == from_node_id,
        })
        .map(|config| config.order_config_id)
        .collect()
}
