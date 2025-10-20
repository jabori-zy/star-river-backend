use star_river_core::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::{
    CustomVariableUpdateOperationValueIsNoneSnafu, DivideByZeroSnafu, UnSupportVariableOperationSnafu,
    BacktestStrategyError,
};
use star_river_core::node::variable_node::variable_operation::UpdateVarValueOperation;
use star_river_core::strategy::custom_variable::VariableValue;


/// Apply variable value update operation
///
/// # Parameters
/// * `var_name` - Variable name
/// * `current_value` - Current variable value
/// * `operation` - Operation type
/// * `operation_value` - Operation value (optional for some operations)
///
/// # Returns
/// Updated variable value or error
pub fn apply_variable_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation: &UpdateVarValueOperation,
    operation_value: Option<&VariableValue>,
) -> Result<VariableValue, BacktestStrategyError> {
    match operation {
        UpdateVarValueOperation::Set => {
            apply_set_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Add => {
            apply_add_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Subtract => {
            apply_subtract_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Multiply => {
            apply_multiply_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Divide => {
            apply_divide_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Max => {
            apply_max_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Min => {
            apply_min_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Toggle => {
            apply_toggle_operation(var_name, current_value, operation)
        }
        UpdateVarValueOperation::Append => {
            apply_append_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Remove => {
            apply_remove_operation(var_name, current_value, operation_value, operation)
        }
        UpdateVarValueOperation::Clear => {
            apply_clear_operation(var_name, current_value, operation)
        }
    }
}

fn apply_set_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(_), VariableValue::Number(v)) => Ok(VariableValue::Number(*v)),
        (VariableValue::Percentage(_), VariableValue::Percentage(v)) => {
            Ok(VariableValue::Percentage(*v))
        }
        (VariableValue::String(_), VariableValue::String(v)) => {
            Ok(VariableValue::String(v.clone()))
        }
        (VariableValue::Time(_), VariableValue::Time(v)) => Ok(VariableValue::Time(*v)),
        (VariableValue::Boolean(_), VariableValue::Boolean(v)) => Ok(VariableValue::Boolean(*v)),
        (VariableValue::Enum(_), VariableValue::Enum(v)) => Ok(VariableValue::Enum(v.clone())),
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_add_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(var_value), VariableValue::Number(op_value)) => {
            Ok(VariableValue::Number(*var_value + *op_value))
        }
        (VariableValue::Percentage(var_value), VariableValue::Percentage(op_value)) => {
            Ok(VariableValue::Percentage(*var_value + *op_value))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_subtract_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(var_value), VariableValue::Number(op_value)) => {
            Ok(VariableValue::Number(*var_value - *op_value))
        }
        (VariableValue::Percentage(var_value), VariableValue::Percentage(op_value)) => {
            Ok(VariableValue::Percentage(*var_value - *op_value))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_multiply_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(var_value), VariableValue::Number(op_value)) => {
            Ok(VariableValue::Number(*var_value * *op_value))
        }
        (VariableValue::Percentage(var_value), VariableValue::Percentage(op_value)) => {
            Ok(VariableValue::Percentage(*var_value * *op_value))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_divide_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(var_value), VariableValue::Number(op_value)) => {
            if op_value.is_zero() {
                return Err(DivideByZeroSnafu {
                    var_name: var_name.to_string(),
                }
                .build());
            }
            Ok(VariableValue::Number(*var_value / *op_value))
        }
        (VariableValue::Percentage(var_value), VariableValue::Percentage(op_value)) => {
            if op_value.is_zero() {
                return Err(DivideByZeroSnafu {
                    var_name: var_name.to_string(),
                }
                .build());
            }
            Ok(VariableValue::Percentage(*var_value / *op_value))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_max_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(var_value), VariableValue::Number(op_value)) => {
            Ok(VariableValue::Number(*var_value.max(op_value)))
        }
        (VariableValue::Percentage(var_value), VariableValue::Percentage(op_value)) => {
            Ok(VariableValue::Percentage(*var_value.max(op_value)))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_min_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Number(var_value), VariableValue::Number(op_value)) => {
            Ok(VariableValue::Number(*var_value.min(op_value)))
        }
        (VariableValue::Percentage(var_value), VariableValue::Percentage(op_value)) => {
            Ok(VariableValue::Percentage(*var_value.min(op_value)))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_toggle_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    match current_value {
        VariableValue::Boolean(var_value) => Ok(VariableValue::Boolean(!var_value)),
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_append_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Enum(var_value), VariableValue::Enum(op_value)) => {
            let mut new_enum = var_value.clone();
            new_enum.extend(op_value.clone());
            Ok(VariableValue::Enum(new_enum))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_remove_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation_value: Option<&VariableValue>,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    let op_value = operation_value.ok_or_else(|| {
        CustomVariableUpdateOperationValueIsNoneSnafu {
            var_name: var_name.to_string(),
        }
        .build()
    })?;

    match (current_value, op_value) {
        (VariableValue::Enum(var_value), VariableValue::Enum(op_value)) => {
            let mut new_enum = var_value.clone();
            new_enum.retain(|item| !op_value.contains(item));
            Ok(VariableValue::Enum(new_enum))
        }
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

fn apply_clear_operation(
    var_name: &str,
    current_value: &VariableValue,
    operation: &UpdateVarValueOperation,
) -> Result<VariableValue, BacktestStrategyError> {
    match current_value {
        VariableValue::Enum(_) => Ok(VariableValue::Enum(Vec::new())),
        _ => Err(UnSupportVariableOperationSnafu {
            var_name: var_name.to_string(),
            var_type: current_value.value_type(),
            operation: operation.to_string(),
        }
        .build()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_set_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Set;
        let operation_value = VariableValue::Number(Decimal::from(20));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(20)));
    }

    #[test]
    fn test_set_operation_percentage() {
        let current = VariableValue::Percentage(Decimal::from_str("0.5").unwrap());
        let operation = UpdateVarValueOperation::Set;
        let operation_value = VariableValue::Percentage(Decimal::from_str("0.75").unwrap());

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Percentage(Decimal::from_str("0.75").unwrap()));
    }

    #[test]
    fn test_set_operation_missing_value() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Set;

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_add_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Add;
        let operation_value = VariableValue::Number(Decimal::from(5));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(15)));
    }

    #[test]
    fn test_add_operation_percentage() {
        let current = VariableValue::Percentage(Decimal::from_str("0.3").unwrap());
        let operation = UpdateVarValueOperation::Add;
        let operation_value = VariableValue::Percentage(Decimal::from_str("0.2").unwrap());

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Percentage(Decimal::from_str("0.5").unwrap()));
    }

    #[test]
    fn test_subtract_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Subtract;
        let operation_value = VariableValue::Number(Decimal::from(3));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(7)));
    }

    #[test]
    fn test_multiply_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Multiply;
        let operation_value = VariableValue::Number(Decimal::from(3));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(30)));
    }

    #[test]
    fn test_divide_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Divide;
        let operation_value = VariableValue::Number(Decimal::from(2));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(5)));
    }

    #[test]
    fn test_divide_by_zero_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Divide;
        let operation_value = VariableValue::Number(Decimal::ZERO);

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_divide_by_zero_percentage() {
        let current = VariableValue::Percentage(Decimal::from_str("0.5").unwrap());
        let operation = UpdateVarValueOperation::Divide;
        let operation_value = VariableValue::Percentage(Decimal::ZERO);

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_max_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Max;
        let operation_value = VariableValue::Number(Decimal::from(15));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(15)));
    }

    #[test]
    fn test_max_operation_current_larger() {
        let current = VariableValue::Number(Decimal::from(20));
        let operation = UpdateVarValueOperation::Max;
        let operation_value = VariableValue::Number(Decimal::from(15));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(20)));
    }

    #[test]
    fn test_min_operation_number() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Min;
        let operation_value = VariableValue::Number(Decimal::from(5));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(5)));
    }

    #[test]
    fn test_min_operation_current_smaller() {
        let current = VariableValue::Number(Decimal::from(3));
        let operation = UpdateVarValueOperation::Min;
        let operation_value = VariableValue::Number(Decimal::from(5));

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::Number(Decimal::from(3)));
    }

    #[test]
    fn test_toggle_operation_true_to_false() {
        let current = VariableValue::Boolean(true);
        let operation = UpdateVarValueOperation::Toggle;

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            None,
        ).unwrap();

        assert_eq!(result, VariableValue::Boolean(false));
    }

    #[test]
    fn test_toggle_operation_false_to_true() {
        let current = VariableValue::Boolean(false);
        let operation = UpdateVarValueOperation::Toggle;

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            None,
        ).unwrap();

        assert_eq!(result, VariableValue::Boolean(true));
    }

    #[test]
    fn test_append_operation_enum() {
        let current = VariableValue::Enum(vec!["a".to_string(), "b".to_string()]);
        let operation = UpdateVarValueOperation::Append;
        let operation_value = VariableValue::Enum(vec!["c".to_string(), "d".to_string()]);

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(
            result,
            VariableValue::Enum(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string()
            ])
        );
    }

    #[test]
    fn test_remove_operation_enum() {
        let current = VariableValue::Enum(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        let operation = UpdateVarValueOperation::Remove;
        let operation_value = VariableValue::Enum(vec!["b".to_string(), "d".to_string()]);

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(
            result,
            VariableValue::Enum(vec!["a".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn test_clear_operation_enum() {
        let current = VariableValue::Enum(vec!["a".to_string(), "b".to_string()]);
        let operation = UpdateVarValueOperation::Clear;

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            None,
        ).unwrap();

        assert_eq!(result, VariableValue::Enum(vec![]));
    }

    #[test]
    fn test_unsupported_operation_type_mismatch() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Set;
        let operation_value = VariableValue::String("test".to_string());

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_toggle_on_non_boolean() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Toggle;

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_clear_on_non_enum() {
        let current = VariableValue::Number(Decimal::from(10));
        let operation = UpdateVarValueOperation::Clear;

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_set_operation_string() {
        let current = VariableValue::String("hello".to_string());
        let operation = UpdateVarValueOperation::Set;
        let operation_value = VariableValue::String("world".to_string());

        let result = apply_variable_operation(
            "test_var",
            &current,
            &operation,
            Some(&operation_value),
        ).unwrap();

        assert_eq!(result, VariableValue::String("world".to_string()));
    }

    #[test]
    fn test_percentage_type_consistency_in_operations() {
        // Test that Percentage operations return Percentage type
        let current = VariableValue::Percentage(Decimal::from_str("0.5").unwrap());

        // Add
        let result = apply_variable_operation(
            "test",
            &current,
            &UpdateVarValueOperation::Add,
            Some(&VariableValue::Percentage(Decimal::from_str("0.2").unwrap())),
        ).unwrap();
        assert!(matches!(result, VariableValue::Percentage(_)));

        // Subtract
        let result = apply_variable_operation(
            "test",
            &current,
            &UpdateVarValueOperation::Subtract,
            Some(&VariableValue::Percentage(Decimal::from_str("0.1").unwrap())),
        ).unwrap();
        assert!(matches!(result, VariableValue::Percentage(_)));

        // Multiply
        let result = apply_variable_operation(
            "test",
            &current,
            &UpdateVarValueOperation::Multiply,
            Some(&VariableValue::Percentage(Decimal::from(2))),
        ).unwrap();
        assert!(matches!(result, VariableValue::Percentage(_)));

        // Divide
        let result = apply_variable_operation(
            "test",
            &current,
            &UpdateVarValueOperation::Divide,
            Some(&VariableValue::Percentage(Decimal::from(2))),
        ).unwrap();
        assert!(matches!(result, VariableValue::Percentage(_)));
    }

    #[test]
    fn test_decimal_precision_no_floating_point_errors() {
        // Test that 0.005 + 1.0 repeated 1000 times gives exactly 1000.005
        let mut current = VariableValue::Number(Decimal::from_str("0.005").unwrap());
        let one = VariableValue::Number(Decimal::ONE);

        for _ in 0..1000 {
            current = apply_variable_operation(
                "test",
                &current,
                &UpdateVarValueOperation::Add,
                Some(&one),
            ).unwrap();
        }

        // Should be exactly 1000.005, no floating point errors
        assert_eq!(current, VariableValue::Number(Decimal::from_str("1000.005").unwrap()));
    }

    #[test]
    fn test_decimal_precision_0_1_plus_0_2() {
        // Classic floating point test: 0.1 + 0.2 should equal 0.3
        let a = VariableValue::Number(Decimal::from_str("0.1").unwrap());
        let b = VariableValue::Number(Decimal::from_str("0.2").unwrap());

        let result = apply_variable_operation(
            "test",
            &a,
            &UpdateVarValueOperation::Add,
            Some(&b),
        ).unwrap();

        // With Decimal, this is exactly 0.3
        assert_eq!(result, VariableValue::Number(Decimal::from_str("0.3").unwrap()));
    }

    #[test]
    fn test_decimal_precision_percentage_operations() {
        // Test percentage precision: 0.05 * 20 should equal 1.0 exactly
        let rate = VariableValue::Percentage(Decimal::from_str("0.05").unwrap());
        let multiplier = VariableValue::Percentage(Decimal::from(20));

        let result = apply_variable_operation(
            "test",
            &rate,
            &UpdateVarValueOperation::Multiply,
            Some(&multiplier),
        ).unwrap();

        assert_eq!(result, VariableValue::Percentage(Decimal::ONE));
    }
}
