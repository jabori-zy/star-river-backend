use std::{collections::HashMap, str::FromStr, sync::Arc};

use rust_decimal::Decimal;
use snafu::{IntoError, ResultExt};
use star_river_core::custom_type::NodeId;
use star_river_event::backtest_strategy::node_event::{
    VariableNodeEvent,
    variable_node_event::{CustomVarUpdateEvent, CustomVarUpdatePayload},
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategyCommandSendFailedSnafu},
    event::node_common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
    node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
    node_infra::variable_node::{
        VariableConfig,
        trigger::{
            TriggerConfig,
            dataflow::{DataFlow, DataflowErrorPolicy, DataflowErrorType},
        },
        variable_config::{get::GetVariableConfig, reset::ResetVariableConfig, update::UpdateVariableConfig},
    },
    variable::{
        custom_variable::VariableValue,
        sys_varibale::{SysVariable, SysVariableType},
    },
};
use tokio::sync::oneshot;

use super::VariableNodeContext;
use crate::{
    node::node_error::{VariableNodeError, variable_node_error::TaskFailedSnafu},
    strategy::strategy_command::{
        ResetCustomVarCmdPayload, ResetCustomVarValueCommand, UpdateCustomVarValueCmdPayload, UpdateCustomVarValueCommand,
    },
};

impl VariableNodeContext {
    pub(super) async fn handle_condition_trigger(
        &mut self,
        condition_trigger_configs: &Vec<VariableConfig>,
    ) -> Result<(), VariableNodeError> {
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
            self.get_variable(&get_var_configs).await?;
        }

        if !update_var_configs.is_empty() {
            self.update_variable(&update_var_configs).await?;
        }

        if !reset_var_configs.is_empty() {
            self.reset_variable(&reset_var_configs).await?;
        }

        Ok(())
    }

    pub(super) async fn handle_dataflow_trigger(
        &mut self,
        dataflow_trigger_configs: &Vec<VariableConfig>,
        dataflow: DataFlow,
    ) -> Result<(), VariableNodeError> {
        let mut update_var_configs = Vec::new();

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
                        if self.is_leaf_node() {
                            self.send_execute_over_event(Some(config.config_id()), Some(self.strategy_time()))?
                        } else {
                            self.send_trigger_event(&output_handle_id, Some(self.strategy_time())).await?;
                        }
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
            self.update_variable(&update_var_configs).await?;
            return Ok(());
        }

        Ok(())
    }

    async fn get_variable(&self, get_var_configs: &Vec<GetVariableConfig>) -> Result<(), VariableNodeError> {
        // 先生成Handler,然后同时执行
        let mut get_var_handles: Vec<tokio::task::JoinHandle<Result<(), VariableNodeError>>> = Vec::with_capacity(get_var_configs.len());

        // 在循环外提前克隆共享数据，避免重复克隆
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let cycle_id = self.cycle_id();
        let current_time = self.strategy_time();
        let strategy_command_sender = self.strategy_command_sender().clone();
        let strategy_output_handle = self.strategy_bound_handle().clone();
        let is_leaf_node = self.is_leaf_node();

        for config in get_var_configs {
            match config {
                GetVariableConfig::Custom(custom_config) => {
                    let output_handle_id = custom_config.output_handle_id.clone();
                    let handle = Self::create_get_custom_var_handle(
                        cycle_id,
                        current_time,
                        node_id.clone(),
                        node_name.clone(),
                        custom_config.config_id(),
                        custom_config.var_name().to_string(),
                        self.output_handle(&output_handle_id).unwrap().clone(),
                        strategy_command_sender.clone(),
                        strategy_output_handle.clone(),
                        is_leaf_node,
                    )
                    .await;
                    get_var_handles.push(handle);
                }
                GetVariableConfig::System(system_config) => {
                    let system_var = SysVariableType::from_str(system_config.var_name()).unwrap();
                    match system_var {
                        SysVariableType::CurrentTime => {
                            let handle = self.create_current_time_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::TotalCurrentPositionAmount => {
                            let handle = self.create_total_current_position_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::CurrentPositionAmount => {
                            let handle = self.create_current_position_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }

                        SysVariableType::TotalUnfilledOrderAmount => {
                            let handle = self.create_total_unfilled_order_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }

                        SysVariableType::UnfilledOrderAmount => {
                            let handle = self.create_unfilled_order_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::TotalHistoryOrderAmount => {
                            let handle = self.create_total_history_order_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::HistoryOrderAmount => {
                            let handle = self.create_history_order_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::HistoryPositionAmount => {
                            let handle = self.create_history_position_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::TotalHistoryPositionAmount => {
                            let handle = self.create_total_history_position_amount_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        SysVariableType::CurrentRoi => {
                            let handle = self.create_current_roi_handle(system_config.clone()).await?;
                            get_var_handles.push(handle);
                        }
                        _ => {}
                    }
                }
            }
        }

        // 等待所有任务完成
        let result = futures::future::join_all(get_var_handles).await;
        result.into_iter().try_for_each(|res| {
            res
            .context(TaskFailedSnafu{})??;
            Ok::<(), VariableNodeError>(())
        })?;
        Ok(())
    }

    async fn update_variable(&self, update_var_configs: &Vec<UpdateVariableConfig>) -> Result<(), VariableNodeError> {
        // 先生成Handler,然后同时执行
        let mut update_handles = Vec::with_capacity(update_var_configs.len());

        // 在循环外提前克隆共享数据，避免重复克隆
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let cycle_id = self.cycle_id();
        let current_time = self.strategy_time();
        let strategy_command_sender = self.strategy_command_sender().clone();
        let strategy_output_handle = self.strategy_bound_handle().clone();
        let is_leaf_node = self.is_leaf_node();

        for config in update_var_configs {
            // 只克隆配置特定的字段
            let var_op = "update".to_string();
            let update_var_value_operation = config.update_var_value_operation().clone();
            let update_operation_value = config.update_operation_value().cloned();

            let config_id = config.config_id();
            let output_handle_id = config.output_handle_id.clone();
            let output_handle = self.output_handle(&output_handle_id)?.clone();
            let update_var_config_clone = config.clone();

            // 为 tokio::spawn 闭包准备克隆的数据
            let node_id_clone = node_id.clone();
            let node_name_clone = node_name.clone();
            let output_handle_id_clone = output_handle_id.clone();
            let sender_clone = strategy_command_sender.clone();
            let strategy_output_handle_clone = strategy_output_handle.clone();

            let handle: tokio::task::JoinHandle<Result<(), VariableNodeError>> = tokio::spawn(async move {
                let (resp_tx, resp_rx) = oneshot::channel();
                let update_var_event = UpdateCustomVarValueCmdPayload::new(update_var_config_clone);
                let cmd = UpdateCustomVarValueCommand::new(node_id_clone.clone(), resp_tx, update_var_event);
                sender_clone.send(cmd.into()).await.map_err(|e| {
                    StrategyCommandSendFailedSnafu {
                        node_name: node_name_clone.clone(),
                    }
                    .into_error(Arc::new(e))
                })?;
                let response = resp_rx.await.unwrap();
                match response {
                    StrategyResponse::Success { payload, .. } => {
                        let payload = CustomVarUpdatePayload::new(
                            cycle_id,
                            config_id,
                            var_op,
                            Some(update_var_value_operation),
                            update_operation_value,
                            payload.custom_variable.clone(),
                        );
                        let var_event: VariableNodeEvent = CustomVarUpdateEvent::new(
                            cycle_id,
                            node_id_clone.clone(),
                            node_name_clone.clone(),
                            output_handle_id_clone.clone(),
                            payload,
                        )
                        .into();
                        strategy_output_handle_clone.send(var_event.clone().into())?;
                        if is_leaf_node {
                            let payload = ExecuteOverPayload::new(None);
                            let execute_over_event: CommonEvent = ExecuteOverEvent::new_with_time(
                                cycle_id,
                                node_id_clone,
                                node_name_clone,
                                output_handle_id_clone,
                                current_time,
                                payload,
                            )
                            .into();
                            strategy_output_handle_clone.send(execute_over_event.into())?;
                        } else {
                            output_handle.send(var_event.into())?;
                        }
                    }
                    StrategyResponse::Fail { error, .. } => {
                        tracing::error!("update_variable failed: {:?}", error);
                        let payload = TriggerPayload;
                        let trigger_event: CommonEvent = TriggerEvent::new_with_time(
                            cycle_id,
                            node_id_clone,
                            node_name_clone,
                            output_handle_id_clone,
                            current_time,
                            payload,
                        )
                        .into();
                        output_handle.send(trigger_event.into())?;
                    }
                }
                Ok(())
            });
            update_handles.push(handle);
        }

        // 等待所有任务完成
        let result = futures::future::join_all(update_handles).await;
        result.into_iter().try_for_each(move |res| {
            res
            .context(TaskFailedSnafu{})??;
            Ok::<(), VariableNodeError>(())
        })?;
        Ok(())
    }

    async fn reset_variable(&self, reset_var_configs: &Vec<ResetVariableConfig>) -> Result<(), VariableNodeError> {
        // 先生成Handler,然后同时执行
        let mut reset_handles = Vec::with_capacity(reset_var_configs.len());

        // 在循环外提前克隆共享数据，避免重复克隆
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let cycle_id = self.cycle_id();
        let current_time = self.strategy_time();
        let strategy_command_sender = self.strategy_command_sender().clone();
        let strategy_output_handle = self.strategy_bound_handle().clone();
        let is_leaf_node = self.is_leaf_node();

        for config in reset_var_configs {
            // 只克隆配置特定的字段
            let var_name = config.var_name().to_string();
            let var_op = "reset".to_string();
            let config_id = config.config_id();

            let output_handle_id = config.output_handle_id.clone();
            let output_handle = self.output_handle(&output_handle_id).unwrap().clone();

            // 为 tokio::spawn 闭包准备克隆的数据
            let node_id_clone = node_id.clone();
            let node_name_clone = node_name.clone();
            let output_handle_id_clone = output_handle_id.clone();
            let sender_clone = strategy_command_sender.clone();
            let strategy_output_handle_clone = strategy_output_handle.clone();

            let handle: tokio::task::JoinHandle<Result<(), VariableNodeError>> = tokio::spawn(async move {
                let (resp_tx, resp_rx) = oneshot::channel();
                let reset_var_event = ResetCustomVarCmdPayload::new(var_name.clone());
                let cmd = ResetCustomVarValueCommand::new(node_id_clone.clone(), resp_tx, reset_var_event);
                sender_clone.send(cmd.into()).await.map_err(|e| {
                    StrategyCommandSendFailedSnafu {
                        node_name: node_name_clone.clone(),
                    }
                    .into_error(Arc::new(e))
                })?;
                let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
                    node_name: node_name_clone.clone(),
                })?;
                match response {
                    StrategyResponse::Success { payload, .. } => {
                        let payload = CustomVarUpdatePayload::new(cycle_id, config_id, var_op, None, None, payload.custom_variable.clone());
                        let var_event: VariableNodeEvent = CustomVarUpdateEvent::new_with_time(
                            cycle_id,
                            node_id_clone.clone(),
                            node_name_clone.clone(),
                            output_handle_id_clone.clone(),
                            current_time,
                            payload,
                        )
                        .into();
                        strategy_output_handle_clone.send(var_event.clone().into())?;
                        if is_leaf_node {
                            let payload = ExecuteOverPayload::new(None);
                            let execute_over_event: CommonEvent = ExecuteOverEvent::new_with_time(
                                cycle_id,
                                node_id_clone,
                                node_name_clone,
                                output_handle_id_clone,
                                current_time,
                                payload,
                            )
                            .into();
                            strategy_output_handle_clone.send(execute_over_event.into())?;
                        } else {
                            output_handle.send(var_event.into())?;
                        }
                    }
                    StrategyResponse::Fail { error, .. } => {
                        tracing::error!("reset_variable failed: {:?}", error);
                        let payload = TriggerPayload;
                        let trigger_event: CommonEvent = TriggerEvent::new_with_time(
                            cycle_id,
                            node_id_clone,
                            node_name_clone,
                            output_handle_id_clone,
                            current_time,
                            payload,
                        )
                        .into();
                        output_handle.send(trigger_event.into())?;
                    }
                }
                Ok(())
            });
            reset_handles.push(handle);
        }

        // 等待所有任务完成
        let result = futures::future::join_all(reset_handles).await;
        result.into_iter().try_for_each(move |res| {
            res
            .context(TaskFailedSnafu{})??;
            Ok::<(), VariableNodeError>(())
        })?;
        Ok(())
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
