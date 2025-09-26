use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::Command;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeResetResponse, StrategyCommand};
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, TriggerEvent, TriggerPayload};
use event_center::event::node_event::backtest_node_event::variable_node_event::{
    SysVariableUpdatedEvent, SysVariableUpdatedPayload, VariableNodeEvent,
};
use event_center::event::node_event::backtest_node_event::{BacktestNodeEvent, IfElseNodeEvent};
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::node::variable_node::GetVariableType;
use star_river_core::node::variable_node::*;

use star_river_core::strategy::sys_varibale::SysVariable;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use virtual_trading::VirtualTradingSystem;

#[derive(Debug, Clone)]
pub struct VariableNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: VariableNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub database: DatabaseConnection,
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
}

#[async_trait]
impl BacktestNodeContextTrait for VariableNodeContext {
    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> &NodeOutputHandle {
        let node_id = self.base_context.node_id.clone();
        self.base_context
            .output_handles
            .get(&format!("{}_default_output", node_id))
            .unwrap()
    }

    async fn handle_engine_event(&mut self, event: Event) {
        tracing::info!("{}: 处理事件: {:?}", self.get_node_id(), event);
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        match node_event {
            BacktestNodeEvent::IfElseNode(IfElseNodeEvent::ConditionMatch(_)) => {
                tracing::debug!("[{}] 条件匹配，获取变量", self.get_node_name());
                // 判断当前节点的模式
                // 如果是条件触发模式，则获取变量
                if self
                    .backtest_config
                    .variable_configs
                    .iter()
                    .any(|v| v.get_variable_type == GetVariableType::Condition)
                {
                    tracing::info!("{}: 条件触发模式，获取变量", self.get_node_name());
                    self.get_variable().await;
                }
            }
            BacktestNodeEvent::Common(CommonEvent::Trigger(_)) => {
                tracing::debug!("{}: 条件不触发模式，不获取变量", self.get_node_name());
                let payload = TriggerPayload::new(self.get_play_index());
                let condition_not_match_event: CommonEvent = TriggerEvent::new(
                    self.base_context.node_id.clone(),
                    self.base_context.node_name.clone(),
                    self.get_default_output_handle().output_handle_id.clone(),
                    payload,
                )
                .into();
                // 获取默认output_handle
                let default_output_handle = self.get_default_output_handle();
                let _ = default_output_handle.send(condition_not_match_event.into());
            }

            _ => {}
        }
    }

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.get_node_id() == &cmd.node_id() {
                    let response = NodeResetResponse::success(self.get_node_id().clone(), None);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}

impl VariableNodeContext {
    pub async fn register_task(&mut self) {
        // let database = self.database.clone();
        // let backtest_config = self.backtest_config.clone();
        // let timer_config = backtest_config.timer_config.unwrap();
        // let variables = backtest_config.variables.clone();
        // let node_name = self.get_node_name().clone();
        // let strategy_id = self.get_strategy_id().clone();
        // let node_id = self.get_node_id().clone();
        // let all_output_handle = self.get_all_output_handle().clone();

        // let mut heartbeat = self.heartbeat.lock().await;
        // heartbeat.register_async_task(format!("{}: 注册处理变量任务", node_name),
        // move || {
        //     let strategy_id = strategy_id.clone();
        //     let node_id = node_id.clone();
        //     let node_name = node_name.clone();
        //     let variables = variables.clone();
        //     let database = database.clone();
        //     let all_output_handle = all_output_handle.clone();
        //     async move {
        //         Self::process_variable(
        //             strategy_id,
        //             node_id,
        //             node_name,
        //             variables,
        //             database,
        //             all_output_handle).await
        //         }
        //     },
        //     timer_config.get_millisecond()/100
        // ).await;
    }

    pub async fn get_variable(&mut self) {
        let variable_configs = self.backtest_config.variable_configs.clone();

        for var_config in variable_configs {
            let variable_type = var_config.variable.clone();
            match variable_type {
                SysVariable::PositionNumber => {
                    let position_number = self.get_position_number(&var_config).await;
                    let payload = SysVariableUpdatedPayload::new(
                        self.get_play_index(),
                        var_config.config_id,
                        var_config.variable.clone(),
                        position_number as f64,
                    );
                    let sys_variable_updated_event: VariableNodeEvent = SysVariableUpdatedEvent::new(
                        self.base_context.node_id.clone(),
                        self.base_context.node_name.clone(),
                        var_config.output_handle_id.clone(),
                        payload,
                    )
                    .into();

                    let output_handle = self.get_output_handle(&var_config.output_handle_id);
                    tracing::debug!("{}: 发送仓位数量更新事件: {:?}", self.get_node_id(), sys_variable_updated_event);
                    let _ = output_handle.send(sys_variable_updated_event.clone().into());

                    let default_output_handle = self.get_default_output_handle();
                    let _ = default_output_handle.send(sys_variable_updated_event.clone().into());

                    let strategy_output_handle = self.get_strategy_output_handle();
                    let _ = strategy_output_handle.send(sys_variable_updated_event.into());
                }
                _ => {}
            }
        }
    }

    async fn process_variable(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        variables: Vec<VariableConfig>,
        database: DatabaseConnection,
        output_handle: HashMap<String, NodeOutputHandle>,
    ) {
        for var in variables {
            let variable_type = var.variable.clone();
            match variable_type {
                SysVariable::PositionNumber => {}
                _ => {}
            }
        }
    }

    async fn get_position_number(&self, variable_config: &VariableConfig) -> u32 {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let exchange = self
            .backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let symbol = variable_config.symbol.clone();

        let current_positions = virtual_trading_system.get_current_positions_ref();
        let position_number = match symbol {
            // 如果symbol不为空，则获取symbol的持仓数量
            Some(symbol) => {
                let position_number = current_positions
                    .iter()
                    .filter(|position| position.symbol == symbol && position.exchange == exchange)
                    .count() as u32;
                position_number
            }
            // 如果symbol为空，则获取所有持仓数量
            None => {
                let position_number = current_positions.len() as u32;
                position_number
            }
        };
        position_number
    }
}
