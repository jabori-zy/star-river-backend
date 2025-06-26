use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp;
use utils::get_utc8_timestamp_millis;
use event_center::strategy_event::StrategyEvent;
use event_center::Event;
use types::strategy::node_event::{SignalEvent, NodeEvent, BacktestConditionMatchEvent, IndicatorEvent, PlayIndexUpdateEvent};
use super::if_else_node_type::IfElseNodeBacktestConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use super::condition::*;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::custom_type::{NodeId, HandleId, VariableId};







// 获取变量值
pub fn get_variable_value(
    node_id: NodeId,
    variable_id: VariableId,
    variable_name: &str,
    received_value: &HashMap<(NodeId, VariableId), Option<NodeEvent>>
) -> Option<f64> {
    
    let node_event = received_value.get(&(node_id, variable_id))?.as_ref()?;
    tracing::debug!("node_event: {:?}", node_event);
    
    match node_event {
        NodeEvent::Indicator(indicator_event) => {
            if let IndicatorEvent::BacktestIndicatorUpdate(indicator_update_event) = indicator_event {
                tracing::debug!("indicator_update_event: {:?}", indicator_update_event);
                indicator_update_event
                    .indicator_series
                    .last()
                    .and_then(|last_indicator| {
                        let indicator_json = last_indicator.to_json();
                        indicator_json.get(variable_name).cloned()
                    })
                    .and_then(|indicator_value| {
                        indicator_value.as_f64().or_else(|| {
                            tracing::warn!("variable '{}'s value '{}' is not a number", variable_name, indicator_value);
                            None
                        })
                    })
            } else {
                None
            }
        }
        NodeEvent::Variable(variable_message) => {
            Some(variable_message.variable_value)
        }
        _ => None
    }
}



// 获取条件中变量的值（从Variable结构体中提取）
pub fn get_condition_variable_value(
    variable: &Variable,
    received_value: &HashMap<(NodeId, VariableId), Option<NodeEvent>>
) -> Option<f64> {
    match variable.var_type {
        VarType::Variable => {
            let node_id = variable.node_id.as_ref()?;
            let variable_id = variable.variable_id?;
            let variable_name = &variable.variable;
            get_variable_value(node_id.clone(), variable_id, variable_name, received_value)
        },
        VarType::Constant => {
            variable.variable.parse::<f64>().ok()
        },
    }
}