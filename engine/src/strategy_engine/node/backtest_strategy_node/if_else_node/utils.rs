use super::condition::*;
use super::if_else_node_context::ConfigId;
use std::collections::HashMap;
use types::custom_type::NodeId;
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use types::strategy::node_event::backtest_node_event::variable_node_event::VariableNodeEvent;
use types::strategy::node_event::{BacktestNodeEvent, IndicatorNodeEvent};

// 获取变量值
pub fn get_variable_value(
    node_id: NodeId,
    variable_id: ConfigId,
    variable_name: &str,
    received_event: &HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>,
) -> Option<f64> {
    let node_event = received_event.get(&(node_id, variable_id))?.as_ref()?;
    // tracing::debug!("node_event: {:?}", node_event);

    match node_event {
        BacktestNodeEvent::IndicatorNode(indicator_event) => {
            if let IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) = indicator_event {
                // tracing::debug!("indicator_update_event: {:?}", indicator_update_event);
                let indicator_value = indicator_update_event
                    .indicator_series
                    .last()
                    .and_then(|last_indicator| {
                        let indicator_json = last_indicator.to_json();
                        indicator_json.get(variable_name).cloned()
                    })
                    .and_then(|indicator_value| {
                        indicator_value.as_f64().or_else(|| {
                            tracing::warn!(
                                "variable '{}'s value '{}' is not a number",
                                variable_name,
                                indicator_value
                            );
                            None
                        })
                    });
                if let Some(indicator_value) = indicator_value {
                    // 如果indicator_value为0，则返回None
                    if indicator_value == 0.0 {
                        return None;
                    }
                }
                indicator_value
            } else {
                None
            }
        }

        BacktestNodeEvent::KlineNode(kline_node_event) => {
            if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_node_event {
                let kline_value = kline_update_event
                    .kline
                    .last()
                    .and_then(|last_kline| {
                        last_kline
                            .to_json()
                            .get(variable_name)
                            .and_then(|value| value.as_f64())
                    })
                    .unwrap_or(0.0);
                if kline_value == 0.0 {
                    return None;
                }
                Some(kline_value)
            } else {
                None
            }
        }
        BacktestNodeEvent::Variable(variable_node_event) => {
            if let VariableNodeEvent::SysVariableUpdated(sys_variable_updated_event) =
                variable_node_event
            {
                Some(sys_variable_updated_event.variable_value)
            } else {
                None
            }
        }
        _ => None,
    }
}

// 获取条件中变量的值（从Variable结构体中提取）
pub fn get_condition_variable_value(
    variable: &Variable,
    received_value: &HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>,
) -> Option<f64> {
    match variable.var_type {
        VarType::Variable => {
            let node_id = variable.node_id.as_ref()?;
            let variable_id = variable.variable_config_id?;
            let variable_name = &variable.variable;
            get_variable_value(node_id.clone(), variable_id, variable_name, received_value)
        }
        VarType::Constant => variable.variable.parse::<f64>().ok(),
    }
}
