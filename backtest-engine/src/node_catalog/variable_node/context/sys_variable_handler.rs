use chrono::Utc;
use rust_decimal::Decimal;
use star_river_core::{custom_type::NodeId, order::OrderStatus};
use star_river_event::backtest_strategy::node_event::{
    VariableNodeEvent,
    variable_node_event::{SysVarUpdateEvent, SysVarUpdatePayload},
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    event::node_common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
    node::context_trait::{NodeCommunicationExt, NodeHandleExt, NodeIdentityExt, NodeRelationExt},
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
    strategy::{
        PlayIndex,
        strategy_command::{UpdateSysVarCmdPayload, UpdateSysVarValueCommand},
    },
    virtual_trading_system::BacktestVts,
};

impl VariableNodeContext {
    /// 通用的系统变量 Handle 创建器
    ///
    /// # 参数
    /// - `play_index`: 播放索引
    /// - `node_id`: 节点ID
    /// - `system_var_config`: 系统变量配置
    /// - `value_calculator`: 计算变量值的异步闭包，接收虚拟交易系统的引用
    async fn create_sys_variable_handle<F>(
        &self,
        play_index: PlayIndex,
        node_id: NodeId,
        system_var_config: GetSystemVariableConfig,
        value_calculator: F,
    ) -> JoinHandle<()>
    where
        F: for<'a> FnOnce(&'a BacktestVts) -> std::pin::Pin<Box<dyn std::future::Future<Output = SysVariable> + Send + 'a>>
            + Send
            + 'static,
    {
        let output_handle = self.output_handle(&system_var_config.output_handle_id()).unwrap().clone();
        let strategy_output_handle = self.strategy_bound_handle().clone();
        let node_name = self.node_name().clone();
        let is_leaf_node = self.is_leaf_node();
        let virtual_trading_system = self.virtual_trading_system.clone();
        let strategy_command_sender = self.strategy_command_sender().clone();
        tokio::spawn(async move {
            // let var_name = SysVariableType::from_str(system_var_config.var_name()).unwrap();

            // 使用异步闭包计算变量值
            let virtual_trading_system_guard = virtual_trading_system.lock().await;
            let sys_variable = value_calculator(&virtual_trading_system_guard).await;
            drop(virtual_trading_system_guard);

            let (resp_tx, resp_rx) = oneshot::channel();
            let update_sys_variable_cmd_payload = UpdateSysVarCmdPayload::new(sys_variable.clone());
            let cmd = UpdateSysVarValueCommand::new(node_id.clone(), resp_tx, update_sys_variable_cmd_payload);
            let _ = strategy_command_sender.send(cmd.into()).await;
            let response = resp_rx.await.unwrap();
            match response {
                StrategyResponse::Success { .. } => {
                    let payload = SysVarUpdatePayload::new(play_index, system_var_config.config_id(), sys_variable);
                    let var_event: VariableNodeEvent = SysVarUpdateEvent::new(
                        node_id.clone(),
                        node_name.clone(),
                        output_handle.output_handle_id().clone(),
                        payload,
                    )
                    .into();
                    let backtest_var_event: BacktestNodeEvent = var_event.clone().into();
                    let _ = strategy_output_handle.send(backtest_var_event.clone());
                    if is_leaf_node {
                        let payload = ExecuteOverPayload::new(play_index as u64, None);
                        let execute_over_event: CommonEvent =
                            ExecuteOverEvent::new(node_id, node_name, output_handle.output_handle_id().clone(), payload).into();
                        let _ = strategy_output_handle.send(execute_over_event.into());
                    } else {
                        let _ = output_handle.send(backtest_var_event);
                    }
                }
                StrategyResponse::Fail { error, .. } => {
                    tracing::error!("update sys variable failed: {:?}", error);
                    let payload = TriggerPayload::new(play_index as u64);
                    let trigger_event: CommonEvent =
                        TriggerEvent::new(node_id, node_name, output_handle.output_handle_id().clone(), payload).into();
                    let backtest_trigger_event: BacktestNodeEvent = trigger_event.into();
                    let _ = output_handle.send(backtest_trigger_event);
                }
            }
        })
    }
    /// 创建获取总持仓数量的 Handle
    pub(super) async fn create_total_position_number_handle(&self, system_var_config: GetSystemVariableConfig) -> JoinHandle<()> {
        let play_index = self.play_index();
        let node_id = self.node_id().clone();
        let var_display_name = system_var_config.var_display_name().clone();
        self.create_sys_variable_handle(play_index, node_id, system_var_config, |vts| {
            Box::pin(async move {
                let current_positions = vts.with_ctx_read(|ctx| ctx.get_current_positions().clone()).await;
                let var_name = SysVariableType::TotalPositionNumber;
                let var_value = VariableValue::Number(Decimal::from(current_positions.len()));
                SysVariable::new(var_name, var_display_name, None, var_value)
            })
        })
        .await
    }

    /// 创建获取总成交订单数量的 Handle
    pub(super) async fn create_total_filled_order_number_handle(&self, system_var_config: GetSystemVariableConfig) -> JoinHandle<()> {
        let play_index = self.play_index();
        let node_id = self.node_id().clone();
        let var_display_name = system_var_config.var_display_name().clone();
        self.create_sys_variable_handle(play_index, node_id, system_var_config, |vts| {
            Box::pin(async move {
                let orders = vts.with_ctx_read(|ctx| ctx.get_orders().clone()).await;
                let filled_order_number = orders
                    .iter()
                    .filter(|order| matches!(order.order_status, OrderStatus::Filled))
                    .count();
                let var_name = SysVariableType::TotalFilledOrderNumber;
                let var_value = VariableValue::Number(Decimal::from(filled_order_number));
                SysVariable::new(var_name, var_display_name, None, var_value)
            })
        })
        .await
    }

    /// 创建获取指定币种成交订单数量的 Handle
    pub(super) async fn create_filled_order_number_handle(
        &self,
        system_var_config: GetSystemVariableConfig,
    ) -> Result<JoinHandle<()>, VariableNodeError> {
        let play_index = self.play_index();
        let node_id = self.node_id().clone();

        // 验证 symbol 不为空
        let symbol = system_var_config.symbol();
        if symbol.is_none() {
            return Err(SysVariableSymbolIsNullSnafu {
                sys_var_name: system_var_config.var_name().to_string(),
            }
            .build());
        }
        let symbol = symbol.clone().unwrap();

        let var_display_name = system_var_config.var_display_name().clone();
        // 调用通用方法，并在闭包中使用捕获的 symbol
        let handle = self
            .create_sys_variable_handle(play_index, node_id, system_var_config, move |vts| {
                Box::pin(async move {
                    let orders = vts.with_ctx_read(|ctx| ctx.get_orders().clone()).await;
                    let filled_order_number = orders
                        .iter()
                        .filter(|order| order.symbol == symbol && matches!(order.order_status, OrderStatus::Filled))
                        .count();
                    let var_name = SysVariableType::FilledOrderNumber;
                    let var_value = VariableValue::Number(Decimal::from(filled_order_number));
                    SysVariable::new(var_name, var_display_name, Some(symbol), var_value)
                })
            })
            .await;

        Ok(handle)
    }

    pub(super) async fn create_current_time_handle(&self, system_var_config: GetSystemVariableConfig) -> JoinHandle<()> {
        let play_index = self.play_index();
        let node_id = self.node_id().clone();

        let var_display_name = system_var_config.var_display_name().clone();
        let handle = self
            .create_sys_variable_handle(play_index, node_id, system_var_config, move |_vts| {
                Box::pin(async move {
                    // let current_time = vts.get_datetime();
                    let current_time = Utc::now();
                    let var_name = SysVariableType::CurrentTime;
                    let var_value = VariableValue::Time(current_time);
                    SysVariable::new(var_name, var_display_name, None, var_value)
                })
            })
            .await;

        handle
    }
}
