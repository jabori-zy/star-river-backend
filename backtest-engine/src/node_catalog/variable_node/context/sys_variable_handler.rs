use std::sync::Arc;

use rust_decimal::Decimal;
use snafu::{IntoError, ResultExt};
use star_river_event::backtest_strategy::node_event::{
    VariableNodeEvent,
    variable_node_event::{SysVarUpdateEvent, SysVarUpdatePayload},
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategyCommandSendFailedSnafu},
    event::node_common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
    node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
    node_infra::variable_node::variable_config::get::GetSystemVariableConfig,
    variable::{
        custom_variable::VariableValue,
        sys_varibale::{SysVariable, SysVariableType},
    },
};
use tokio::{sync::oneshot, task::JoinHandle};
use virtual_trading::vts_trait::VtsCtxAccessor;

use super::VariableNodeContext;
use crate::{
    node::node_error::{VariableNodeError, variable_node_error::SysVariableSymbolIsNullSnafu},
    node_catalog::variable_node::context::BacktestNodeEvent,
    strategy::strategy_command::{UpdateSysVarCmdPayload, UpdateSysVarValueCommand},
    virtual_trading_system::BacktestVts,
};

impl VariableNodeContext {
    /// Generic system variable handle creator
    ///
    /// # Parameters
    /// - `play_index`: Play index
    /// - `node_id`: Node ID
    /// - `system_var_config`: System variable configuration
    /// - `value_calculator`: Async closure to calculate variable value, receives virtual trading system reference
    async fn create_sys_variable_handle<F>(
        &self,
        system_var_config: GetSystemVariableConfig,
        value_calculator: F,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError>
    where
        F: for<'a> FnOnce(&'a BacktestVts) -> std::pin::Pin<Box<dyn std::future::Future<Output = SysVariable> + Send + 'a>>
            + Send
            + 'static,
    {
        let cycle_id = self.cycle_id();
        let node_id = self.node_id().clone();
        let output_handle = self.output_handle(&system_var_config.output_handle_id())?.clone();
        let default_output_handle = self.default_output_handle()?.clone();
        let strategy_output_handle = self.strategy_bound_handle().clone();
        let node_name = self.node_name().clone();
        let is_leaf_node = self.is_leaf_node();
        let virtual_trading_system = self.virtual_trading_system.clone();
        let strategy_command_sender = self.strategy_command_sender().clone();
        let current_time = self.strategy_time();
        let config_id = system_var_config.config_id();
        let handle = tokio::spawn(async move {
            // let var_name = SysVariableType::from_str(system_var_config.var_name()).unwrap();

            // Calculate variable value using async closure
            let vts = virtual_trading_system.as_ref();
            let sys_variable = value_calculator(vts).await;

            let (resp_tx, resp_rx) = oneshot::channel();
            let update_sys_variable_cmd_payload = UpdateSysVarCmdPayload::new(sys_variable.clone());
            let cmd = UpdateSysVarValueCommand::new(node_id.clone(), resp_tx, update_sys_variable_cmd_payload);
            strategy_command_sender.send(cmd.into()).await.map_err(|e| {
                StrategyCommandSendFailedSnafu {
                    node_name: node_name.clone(),
                }
                .into_error(Arc::new(e))
            })?;
            let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
                node_name: node_name.clone(),
            })?;
            match response {
                StrategyResponse::Success { .. } => {
                    let payload = SysVarUpdatePayload::new(cycle_id, config_id, sys_variable);
                    let var_update_evt: VariableNodeEvent = SysVarUpdateEvent::new_with_time(
                        cycle_id,
                        node_id.clone(),
                        node_name.clone(),
                        output_handle.output_handle_id().clone(),
                        current_time,
                        payload,
                    )
                    .into();
                    let backtest_var_event: BacktestNodeEvent = var_update_evt.into();
                    strategy_output_handle.send(backtest_var_event.clone())?;
                    if is_leaf_node {
                        let payload = ExecuteOverPayload::new(Some(config_id), Some("handle update sys variable".to_string()));
                        let execute_over_event: CommonEvent = ExecuteOverEvent::new_with_time(
                            cycle_id,
                            node_id,
                            node_name,
                            output_handle.output_handle_id().clone(),
                            current_time,
                            payload,
                        )
                        .into();
                        strategy_output_handle.send(execute_over_event.into())?;
                    } else {
                        output_handle.send(backtest_var_event.clone())?;
                        default_output_handle.send(backtest_var_event.clone())?;
                    }
                }
                StrategyResponse::Fail { error, .. } => {
                    tracing::error!("update sys variable failed: {:?}", error);
                    let payload = TriggerPayload::new(config_id, Some("handle update sys variable failed".to_string()));
                    let trigger_event: CommonEvent = TriggerEvent::new_with_time(
                        cycle_id,
                        node_id,
                        node_name,
                        output_handle.output_handle_id().clone(),
                        current_time,
                        payload,
                    )
                    .into();
                    let backtest_trigger_event: BacktestNodeEvent = trigger_event.into();
                    output_handle.send(backtest_trigger_event)?;
                }
            }
            Ok(())
        });
        Ok(handle)
    }
    /// Create handle to get total current position amount
    pub(super) async fn create_total_current_position_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let current_positions = vts.with_ctx_read(|ctx| ctx.current_positions_count()).await;
                    let var_name = SysVariableType::TotalCurrentPositionAmount;
                    let var_value = VariableValue::Number(Decimal::from(current_positions));
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    pub(super) async fn create_current_position_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let symbol = system_var_config
            .symbol()
            .as_ref()
            .ok_or_else(|| {
                SysVariableSymbolIsNullSnafu {
                    sys_var_name: system_var_config.var_name().to_string(),
                }
                .build()
            })?
            .to_string();
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let current_positions = vts
                        .with_ctx_read(|ctx| ctx.current_positions_count_of_symbol(&symbol, &exchange))
                        .await;
                    let var_name = SysVariableType::CurrentPositionAmount;
                    let var_value = VariableValue::Number(Decimal::from(current_positions));
                    SysVariable::new(var_name, var_display_name, Some(symbol), var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    /// Create handle to get total unfilled order amount
    pub(super) async fn create_total_unfilled_order_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let unfilled_order_number = vts.with_ctx_read(|ctx| ctx.unfilled_order_count()).await;
                    let var_name = SysVariableType::TotalUnfilledOrderAmount;
                    let var_value = VariableValue::Number(Decimal::from(unfilled_order_number));
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    /// Create handle to get unfilled order amount for specified symbol
    pub(super) async fn create_unfilled_order_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        // Validate symbol is not null
        let symbol = system_var_config
            .symbol()
            .as_ref()
            .ok_or_else(|| {
                SysVariableSymbolIsNullSnafu {
                    sys_var_name: system_var_config.var_name().to_string(),
                }
                .build()
            })?
            .to_string();
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();

        let var_display_name = system_var_config.var_display_name().clone();
        // Call generic method and use captured symbol in closure
        let handle = self
            .create_sys_variable_handle(system_var_config, move |vts| {
                Box::pin(async move {
                    let unfilled_order_number = vts
                        .with_ctx_read(|ctx| ctx.unfilled_order_count_of_symbol(&symbol, &exchange))
                        .await;
                    let var_name = SysVariableType::UnfilledOrderAmount;
                    let var_value = VariableValue::Number(Decimal::from(unfilled_order_number));
                    SysVariable::new(var_name, var_display_name, Some(symbol), var_value)
                })
            })
            .await?;

        Ok(handle)
    }

    pub(super) async fn create_total_history_order_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let history_order_number = vts.with_ctx_read(|ctx| ctx.history_order_count()).await;
                    let var_name = SysVariableType::TotalHistoryOrderAmount;
                    let var_value = VariableValue::Number(Decimal::from(history_order_number));
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    pub(super) async fn create_history_order_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let symbol = system_var_config
            .symbol()
            .as_ref()
            .ok_or_else(|| {
                SysVariableSymbolIsNullSnafu {
                    sys_var_name: system_var_config.var_name().to_string(),
                }
                .build()
            })?
            .to_string();
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let history_order_number = vts.with_ctx_read(|ctx| ctx.history_order_count_of_symbol(&symbol, &exchange)).await;
                    let var_name = SysVariableType::HistoryOrderAmount;
                    let var_value = VariableValue::Number(Decimal::from(history_order_number));
                    SysVariable::new(var_name, var_display_name, Some(symbol), var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    pub(super) async fn create_total_history_position_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let history_position_amount = vts.with_ctx_read(|ctx| ctx.history_positions_count()).await;
                    let var_name = SysVariableType::TotalHistoryPositionAmount;
                    let var_value = VariableValue::Number(Decimal::from(history_position_amount));
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    pub(super) async fn create_history_position_amount_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let symbol = system_var_config
            .symbol()
            .as_ref()
            .ok_or_else(|| {
                SysVariableSymbolIsNullSnafu {
                    sys_var_name: system_var_config.var_name().to_string(),
                }
                .build()
            })?
            .to_string();
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let history_position_amount = vts
                        .with_ctx_read(|ctx| ctx.history_positions_count_of_symbol(&symbol, &exchange))
                        .await;
                    let var_name = SysVariableType::HistoryPositionAmount;
                    let var_value = VariableValue::Number(Decimal::from(history_position_amount));
                    SysVariable::new(var_name, var_display_name, Some(symbol), var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    pub(super) async fn create_current_roi_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let symbol = system_var_config
            .symbol()
            .as_ref()
            .ok_or_else(|| {
                SysVariableSymbolIsNullSnafu {
                    sys_var_name: system_var_config.var_name().to_string(),
                }
                .build()
            })?
            .to_string();
        let exchange = self.node_config.exchange_mode()?.selected_account.exchange.clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, |vts| {
                Box::pin(async move {
                    let current_roi = vts
                        .with_ctx_read(|ctx| ctx.find_position_for(&symbol, &exchange).map(|p| p.roi))
                        .await;
                    let var_name = SysVariableType::CurrentRoi;
                    let var_value = current_roi
                        .map(|roi| VariableValue::percentage(roi * 100.0))
                        .unwrap_or(VariableValue::Null);
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await?;
        Ok(handle)
    }

    pub(super) async fn create_current_time_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<Result<(), VariableNodeError>>, VariableNodeError> {
        let var_display_name = system_var_config.var_display_name().clone();
        let handle = self
            .create_sys_variable_handle(system_var_config, move |vts| {
                Box::pin(async move {
                    // let current_time = vts.get_datetime();
                    let current_time = vts.with_ctx_read(|ctx| ctx.current_datetime()).await;
                    let var_name = SysVariableType::CurrentTime;
                    let var_value = VariableValue::Time(current_time);
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await?;

        Ok(handle)
    }
}
