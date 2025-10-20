use star_river_core::node::if_else_node::*;
use super::if_else_node_context::ConfigId;
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::event::node_event::backtest_node_event::variable_node_event::VariableNodeEvent;
use event_center::event::node_event::backtest_node_event::{BacktestNodeEvent, IndicatorNodeEvent};
use star_river_core::custom_type::NodeId;
use star_river_core::market::QuantData;
use std::collections::HashMap;
use star_river_core::strategy::custom_variable::VariableValue;
// 获取变量值
pub fn get_variable_value(
    node_id: NodeId,
    variable_id: ConfigId,
    variable_name: &str,
    received_event: &HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>,
) -> VariableValue {
    let Some(Some(node_event)) = received_event.get(&(node_id, variable_id)) else {
        return VariableValue::Null;
    };
    // tracing::debug!("node_event: {:?}", node_event);

    match node_event {
        BacktestNodeEvent::IndicatorNode(indicator_event) => {
            use rust_decimal::Decimal;
            let IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) = indicator_event;
            // tracing::debug!("indicator_update_event: {:?}", indicator_update_event);
            indicator_update_event
                .indicator_value
                .to_json()
                .get(variable_name)
                .and_then(|indicator_value| indicator_value.as_f64())
                .and_then(|v| Decimal::try_from(v).ok())
                .map(VariableValue::Number)
                .unwrap_or(VariableValue::Null)
            }

        BacktestNodeEvent::KlineNode(kline_node_event) => {
            if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_node_event {
                use rust_decimal::Decimal;
                kline_update_event
                    .kline
                    .to_json()
                    .get(variable_name)
                    .and_then(|value| value.as_f64())
                    .filter(|&v| v != 0.0)
                    .and_then(|v| Decimal::try_from(v).ok())
                    .map(VariableValue::Number)
                    .unwrap_or(VariableValue::Null)
            } else {
                VariableValue::Null
            }
        }
        // BacktestNodeEvent::VariableNode(variable_node_event) => {
        //     if let VariableNodeEvent::SysVariableUpdated(sys_variable_updated_event) = variable_node_event {
        //         Some(sys_variable_updated_event.variable_value)
        //     } else {
        //         None
        //     }
        // }
        _ => VariableValue::Null,
    }
}

// 获取条件中变量的值（从Variable结构体中提取）
pub fn get_condition_left_value(
    left: &Variable,
    received_value: &HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>,
) -> VariableValue {
    let node_id = left.node_id.clone();
    let variable_id = left.var_config_id;
    let variable_name = &left.var_name;
    get_variable_value(node_id.clone(), variable_id, variable_name, received_value)

}


pub fn get_condition_right_value(
    right: &FormulaRight,
    received_value: &HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>,
) -> VariableValue {
    match right {
        FormulaRight::Variable(variable) => {
            let node_id = variable.node_id.clone();
            let variable_id = variable.var_config_id;
            let variable_name = &variable.var_name;
            get_variable_value(node_id.clone(), variable_id, variable_name, received_value)
        }
        FormulaRight::Constant(constant) => constant.var_value.clone(),
    }
}



pub fn compare(left: &VariableValue, right: &VariableValue, comparison_symbol: &ComparisonSymbol) -> bool {
    match (left, right) {
        // number和number比较
        (VariableValue::Number(left_value), VariableValue::Number(right_value)) => {
            match comparison_symbol {
                ComparisonSymbol::GreaterThan => left_value > right_value, //>
                ComparisonSymbol::LessThan => left_value < right_value, //<
                ComparisonSymbol::Equal => left_value == right_value, // Decimal精确比较
                ComparisonSymbol::GreaterThanOrEqual => left_value >= right_value, //>=
                ComparisonSymbol::LessThanOrEqual => left_value <= right_value, //<=
                ComparisonSymbol::NotEqual => left_value != right_value,
                _ => false,
            }
        }

        // string和string比较
        (VariableValue::String(left_value), VariableValue::String(right_value)) => {
            match comparison_symbol {
                ComparisonSymbol::Is => left_value == right_value,
                ComparisonSymbol::IsNot => left_value != right_value,
                ComparisonSymbol::Contains => left_value.contains(right_value.as_str()), // A包含B
                ComparisonSymbol::NotContains => !left_value.contains(right_value.as_str()),
                ComparisonSymbol::IsEmpty => left_value.is_empty(),
                ComparisonSymbol::IsNotEmpty => !left_value.is_empty(),
                _ => false,
            }
        }

        (VariableValue::Boolean(left_value), VariableValue::Boolean(right_value)) => {
            match comparison_symbol {
                ComparisonSymbol::Is => left_value == right_value,
                ComparisonSymbol::IsNot => left_value != right_value,
                _ => false,
            }
        }

        (VariableValue::Percentage(left_value), VariableValue::Percentage(right_value)) => {
            match comparison_symbol {
                ComparisonSymbol::GreaterThan => left_value > right_value,
                ComparisonSymbol::LessThan => left_value < right_value,
                ComparisonSymbol::Equal => left_value == right_value, // Decimal精确比较
                ComparisonSymbol::GreaterThanOrEqual => left_value >= right_value,
                ComparisonSymbol::LessThanOrEqual => left_value <= right_value,
                ComparisonSymbol::NotEqual => left_value != right_value,
                _ => false,
            }
        }

        (VariableValue::Time(left_value), VariableValue::Time(right_value)) => {
            match comparison_symbol {
                ComparisonSymbol::GreaterThan => left_value > right_value,
                ComparisonSymbol::LessThan => left_value < right_value,
                ComparisonSymbol::Equal => left_value == right_value,
                ComparisonSymbol::GreaterThanOrEqual => left_value >= right_value,
                ComparisonSymbol::LessThanOrEqual => left_value <= right_value,
                ComparisonSymbol::NotEqual => left_value != right_value,
                _ => false,
            }
        }

        _ => false,
    }
}