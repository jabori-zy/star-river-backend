use super::{
    BacktestNodeContextTrait, DataFlow, GetVariableConfig, NodeId, ResetVariableConfig, Response, TriggerConfig, UpdateVariableConfig,
    VariableConfig, VariableNodeContext, VariableValue,
};

use event_center::{
    communication::backtest_strategy::{
        GetCustomVariableValueCmdPayload, GetCustomVariableValueCommand, ResetCustomVariableValueCmdPayload,
        ResetCustomVariableValueCommand, UpdateCustomVariableValueCmdPayload, UpdateCustomVariableValueCommand,
    },
    event::node_event::backtest_node_event::{
        CommonEvent, VariableNodeEvent,
        common_event::{ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
        variable_node_event::{CustomVariableUpdateEvent, CustomVariableUpdatePayload},
    },
};
use rust_decimal::Decimal;
use star_river_core::{
    market::QuantData,
    node::variable_node::trigger::dataflow::{DataflowErrorPolicy, DataflowErrorType},
};
use std::collections::HashMap;
use tokio::sync::oneshot;

impl VariableNodeContext {
    pub(super) async fn handle_condition_trigger(&mut self, condition_trigger_configs: &Vec<VariableConfig>) {
        // 分成get, update, reset三批
        let get_var_configs = condition_trigger_configs
            .iter()
            .filter_map(|config| match config {
                VariableConfig::Get(get_config) => Some(get_config.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let update_var_configs = condition_trigger_configs
            .iter()
            .filter_map(|config| match config {
                VariableConfig::Update(update_config) => Some(update_config.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let reset_var_configs = condition_trigger_configs
            .iter()
            .filter_map(|config| match config {
                VariableConfig::Reset(reset_config) => Some(reset_config.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        if !get_var_configs.is_empty() {
            self.get_variable(&get_var_configs).await;
        }

        if !update_var_configs.is_empty() {
            self.update_variable(&update_var_configs).await;
        }

        if !reset_var_configs.is_empty() {
            self.reset_variable(&reset_var_configs).await;
        }
    }

    pub(super) async fn handle_dataflow_trigger(&mut self, dataflow_trigger_configs: &Vec<VariableConfig>, dataflow: DataFlow) {
        let mut update_var_configs = Vec::new();

        let play_index = self.get_play_index();
        let node_id = self.get_node_id().clone();
        let node_name = self.get_node_name().clone();

        for config in dataflow_trigger_configs {
            // 使用 if let 链式语法
            if let VariableConfig::Update(mut update_var_config) = config.clone()
                && let TriggerConfig::Dataflow(dataflow_trigger) = update_var_config.trigger_config()
            {
                let from_var_name = dataflow_trigger.from_var.clone();
                let from_var_node_id = dataflow_trigger.from_node_id.clone();
                let from_var_config_id = dataflow_trigger.from_var_config_id;

                let output_handle_id = update_var_config.output_handle_id.clone();

                let value = match (dataflow_trigger.from_node_type.as_str(), &dataflow) {
                    ("klineNode", DataFlow::Kline(kline)) => kline
                        .get_value(&from_var_name)
                        .and_then(|value| Decimal::try_from(value).ok())
                        .map(VariableValue::Number)
                        .unwrap_or(VariableValue::Null),

                    ("indicatorNode", DataFlow::Indicator(indicator)) => indicator
                        .get_value(&from_var_name)
                        .and_then(|value| Decimal::try_from(value).ok())
                        .map(VariableValue::Number)
                        .unwrap_or(VariableValue::Null),

                    _ => VariableValue::Null,
                };

                let error_policy = dataflow_trigger.error_policy.clone();

                // 使用 handle_error_value 方法处理错误值
                let result_value = match self
                    .handle_error_value(value, &error_policy, from_var_node_id.clone(), from_var_config_id, &from_var_name)
                    .await
                {
                    Some(val) => val,
                    None => {
                        // 发送trigger事件
                        let payload = TriggerPayload::new(play_index);
                        let trigger_event: CommonEvent =
                            TriggerEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into();
                        let _ = self.get_output_handle(&output_handle_id).send(trigger_event.into());
                        continue;
                    } // 如果返回 None，跳过当前迭代
                };

                self.update_variable_cache_value(
                    from_var_node_id.clone(),
                    from_var_config_id,
                    from_var_name.clone(),
                    result_value.clone(),
                )
                .await;
                update_var_config.update_operation_value = Some(result_value);
                update_var_configs.push(update_var_config);
            }
        }

        if !update_var_configs.is_empty() {
            self.update_variable(&update_var_configs).await;
        }
    }

    async fn get_variable(&self, get_var_configs: &Vec<GetVariableConfig>) {
        // 先生成Handler,然后同时执行
        let mut get_handles = Vec::with_capacity(get_var_configs.len());

        // 在循环外提前克隆共享数据，避免重复克隆
        let node_id = self.get_node_id();
        let node_name = self.get_node_name();
        let play_index = self.get_play_index();
        let strategy_command_sender = self.get_strategy_command_sender().clone();
        let strategy_output_handle = self.get_strategy_output_handle().clone();
        let is_leaf_node = self.is_leaf_node();

        for config in get_var_configs {
            if let GetVariableConfig::Custom(custom_config) = config {
                // 只克隆配置特定的字段
                let var_name = custom_config.var_name().to_string();
                let config_id = custom_config.config_id();
                let output_handle_id = custom_config.output_handle_id.clone();
                let output_handle = self.get_output_handle(&output_handle_id).clone();

                // 为 tokio::spawn 闭包准备克隆的数据
                let node_id_clone = node_id.clone();
                let node_name_clone = node_name.clone();
                let output_handle_id_clone = output_handle_id.clone();
                let sender_clone = strategy_command_sender.clone();
                let strategy_output_handle_clone = strategy_output_handle.clone();

                let handle = tokio::spawn(async move {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let get_custom_var_event = GetCustomVariableValueCmdPayload::new(var_name.clone());
                    let cmd = GetCustomVariableValueCommand::new(node_id_clone.clone(), resp_tx, Some(get_custom_var_event));
                    sender_clone.send(cmd.into()).await.unwrap();
                    let response = resp_rx.await.unwrap();

                    if response.is_success() {
                        let payload = CustomVariableUpdatePayload::new(
                            play_index,
                            node_id_clone.clone(),
                            config_id,
                            var_name,
                            response.var_value.clone(),
                        );
                        let var_event: VariableNodeEvent = CustomVariableUpdateEvent::new(
                            node_id_clone.clone(),
                            node_name_clone.clone(),
                            output_handle_id_clone.clone(),
                            payload,
                        )
                        .into();

                        // 发送到策略
                        let _ = strategy_output_handle_clone.send(var_event.clone().into());

                        // 如果是叶子节点，则发送执行结束事件
                        if is_leaf_node {
                            let payload = ExecuteOverPayload::new(play_index);
                            let execute_over_event: CommonEvent =
                                ExecuteOverEvent::new(node_id_clone, node_name_clone, output_handle_id_clone, payload).into();
                            let _ = strategy_output_handle_clone.send(execute_over_event.into());
                        } else {
                            let _ = output_handle.send(var_event.into());
                        }
                    } else {
                        tracing::error!("get_variable failed: {:?}", response.get_error());
                        // 失败，发送触发事件
                        let payload = TriggerPayload::new(play_index);
                        let trigger_event: CommonEvent =
                            TriggerEvent::new(node_id_clone, node_name_clone, output_handle_id_clone, payload).into();
                        let _ = output_handle.send(trigger_event.into());
                    }
                });
                get_handles.push(handle);
            }
        }

        // 等待所有任务完成
        futures::future::join_all(get_handles).await;
    }

    async fn update_variable(&self, update_var_configs: &Vec<UpdateVariableConfig>) {
        // 先生成Handler,然后同时执行
        let mut update_handles = Vec::with_capacity(update_var_configs.len());

        // 在循环外提前克隆共享数据，避免重复克隆
        let node_id = self.get_node_id();
        let node_name = self.get_node_name();
        let play_index = self.get_play_index();
        let strategy_command_sender = self.get_strategy_command_sender().clone();
        let strategy_output_handle = self.get_strategy_output_handle().clone();
        let is_leaf_node = self.is_leaf_node();

        for config in update_var_configs {
            // 只克隆配置特定的字段
            let var_name = config.var_name().to_string();
            let config_id = config.config_id();
            let output_handle_id = config.output_handle_id.clone();
            let output_handle = self.get_output_handle(&output_handle_id).clone();
            let update_var_config_clone = config.clone();

            // 为 tokio::spawn 闭包准备克隆的数据
            let node_id_clone = node_id.clone();
            let node_name_clone = node_name.clone();
            let output_handle_id_clone = output_handle_id.clone();
            let sender_clone = strategy_command_sender.clone();
            let strategy_output_handle_clone = strategy_output_handle.clone();

            let handle = tokio::spawn(async move {
                let (resp_tx, resp_rx) = oneshot::channel();
                let update_var_event = UpdateCustomVariableValueCmdPayload::new(update_var_config_clone);
                let cmd = UpdateCustomVariableValueCommand::new(node_id_clone.clone(), resp_tx, Some(update_var_event));
                let _ = sender_clone.send(cmd.into()).await;
                let response = resp_rx.await.unwrap();

                if response.is_success() {
                    let payload = CustomVariableUpdatePayload::new(
                        play_index,
                        node_id_clone.clone(),
                        config_id,
                        var_name,
                        response.var_value.clone(),
                    );
                    let var_event: VariableNodeEvent = CustomVariableUpdateEvent::new(
                        node_id_clone.clone(),
                        node_name_clone.clone(),
                        output_handle_id_clone.clone(),
                        payload,
                    )
                    .into();

                    // 发送到策略
                    let _ = strategy_output_handle_clone.send(var_event.clone().into());

                    // 如果是叶子节点，则发送执行结束事件
                    if is_leaf_node {
                        let payload = ExecuteOverPayload::new(play_index);
                        let execute_over_event: CommonEvent =
                            ExecuteOverEvent::new(node_id_clone, node_name_clone, output_handle_id_clone, payload).into();
                        let _ = strategy_output_handle_clone.send(execute_over_event.into());
                    } else {
                        let _ = output_handle.send(var_event.into());
                    }
                } else {
                    tracing::error!("update_variable failed: {:?}", response.get_error());
                    // 失败，发送触发事件
                    let payload = TriggerPayload::new(play_index);
                    let trigger_event: CommonEvent =
                        TriggerEvent::new(node_id_clone, node_name_clone, output_handle_id_clone, payload).into();
                    let _ = output_handle.send(trigger_event.into());
                }
            });
            update_handles.push(handle);
        }

        // 等待所有任务完成
        futures::future::join_all(update_handles).await;
    }

    async fn reset_variable(&self, reset_var_configs: &Vec<ResetVariableConfig>) {
        // 先生成Handler,然后同时执行
        let mut reset_handles = Vec::with_capacity(reset_var_configs.len());

        // 在循环外提前克隆共享数据，避免重复克隆
        let node_id = self.get_node_id();
        let node_name = self.get_node_name();
        let play_index = self.get_play_index();
        let strategy_command_sender = self.get_strategy_command_sender().clone();
        let strategy_output_handle = self.get_strategy_output_handle().clone();
        let is_leaf_node = self.is_leaf_node();

        for config in reset_var_configs {
            // 只克隆配置特定的字段
            let var_name = config.var_name().to_string();
            let config_id = config.config_id();
            let output_handle_id = config.output_handle_id.clone();
            let output_handle = self.get_output_handle(&output_handle_id).clone();

            // 为 tokio::spawn 闭包准备克隆的数据
            let node_id_clone = node_id.clone();
            let node_name_clone = node_name.clone();
            let output_handle_id_clone = output_handle_id.clone();
            let sender_clone = strategy_command_sender.clone();
            let strategy_output_handle_clone = strategy_output_handle.clone();

            let handle = tokio::spawn(async move {
                let (resp_tx, resp_rx) = oneshot::channel();
                let reset_var_event = ResetCustomVariableValueCmdPayload::new(var_name.clone());
                let cmd = ResetCustomVariableValueCommand::new(node_id_clone.clone(), resp_tx, Some(reset_var_event));
                let _ = sender_clone.send(cmd.into()).await;
                let response = resp_rx.await.unwrap();

                if response.is_success() {
                    let payload = CustomVariableUpdatePayload::new(
                        play_index,
                        node_id_clone.clone(),
                        config_id,
                        var_name,
                        response.initial_value.clone(),
                    );
                    let var_event: VariableNodeEvent = CustomVariableUpdateEvent::new(
                        node_id_clone.clone(),
                        node_name_clone.clone(),
                        output_handle_id_clone.clone(),
                        payload,
                    )
                    .into();

                    // 发送到策略
                    let _ = strategy_output_handle_clone.send(var_event.clone().into());

                    // 如果是叶子节点，则发送执行结束事件
                    if is_leaf_node {
                        let payload = ExecuteOverPayload::new(play_index);
                        let execute_over_event: CommonEvent =
                            ExecuteOverEvent::new(node_id_clone, node_name_clone, output_handle_id_clone, payload).into();
                        let _ = strategy_output_handle_clone.send(execute_over_event.into());
                    } else {
                        let _ = output_handle.send(var_event.into());
                    }
                } else {
                    tracing::error!("reset_variable failed: {:?}", response.get_error());
                    // 失败，发送触发事件
                    let payload = TriggerPayload::new(play_index);
                    let trigger_event: CommonEvent =
                        TriggerEvent::new(node_id_clone, node_name_clone, output_handle_id_clone, payload).into();
                    let _ = output_handle.send(trigger_event.into());
                }
            });
            reset_handles.push(handle);
        }

        // 等待所有任务完成
        futures::future::join_all(reset_handles).await;
    }

    /// 处理错误值的策略
    /// 返回 None 表示应该跳过，返回 Some(value) 表示应该使用的值
    async fn handle_error_value(
        &mut self,
        value: VariableValue,
        error_policy: &HashMap<DataflowErrorType, DataflowErrorPolicy>,
        node_id: NodeId,
        config_id: i32,
        var_name: &String,
    ) -> Option<VariableValue> {
        match value {
            VariableValue::Null => {
                let null_value_policy = error_policy.get(&DataflowErrorType::NullValue).unwrap();
                match null_value_policy {
                    DataflowErrorPolicy::Skip(_policy) => None,
                    DataflowErrorPolicy::UsePreviousValue(_policy) => {
                        let previous_value = self.get_variable_cache_value(node_id, config_id, var_name.clone()).await;
                        if let Some(previous_value) = previous_value {
                            Some(previous_value)
                        } else {
                            // 如果previous_value为None，则跳过
                            None
                        }
                    }
                    DataflowErrorPolicy::ValueReplace(policy) => {
                        // 替换为指定值
                        Some(policy.replace_value.clone())
                    }
                    _ => None,
                }
            }
            VariableValue::Number(v) => {
                // 零值处理
                if v.is_zero() {
                    let zero_value_policy = error_policy.get(&DataflowErrorType::ZeroValue).unwrap();
                    match zero_value_policy {
                        DataflowErrorPolicy::Skip(_policy) => None,
                        DataflowErrorPolicy::UsePreviousValue(_policy) => {
                            let previous_value = self.get_variable_cache_value(node_id, config_id, var_name.clone()).await;
                            if let Some(previous_value) = previous_value {
                                Some(previous_value)
                            } else {
                                // 如果previous_value为None，则跳过
                                None
                            }
                        }
                        DataflowErrorPolicy::ValueReplace(policy) => Some(policy.replace_value.clone()),
                        DataflowErrorPolicy::StillUpdate(_policy) => Some(value),
                    }
                } else {
                    Some(value)
                }
            }
            _ => Some(value),
        }
    }
}
