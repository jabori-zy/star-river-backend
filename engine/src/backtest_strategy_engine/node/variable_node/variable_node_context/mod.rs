mod context_impl;
mod context_utils;
mod variable_handler;

use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_types::NodeOutputHandle;
use event_center::communication::Command;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::{BacktestNodeCommand, NodeResetResponse};
use event_center::event::Event;
use event_center::event::node_event::backtest_node_event::if_else_node_event::ConditionMatchEvent;
use event_center::event::node_event::backtest_node_event::{BacktestNodeEvent, IfElseNodeEvent};
use event_center::event::node_event::NodeEventTrait;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::NodeId;
use star_river_core::node::variable_node::trigger::dataflow::DataFlow;
use star_river_core::node::variable_node::trigger::{ConditionTrigger, TriggerConfig};
use star_river_core::node::variable_node::variable_config::{GetVariableConfig, ResetVariableConfig, UpdateVariableConfig};
use star_river_core::node::variable_node::*;
use star_river_core::strategy::custom_variable::VariableValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use virtual_trading::VirtualTradingSystem;

#[derive(Debug, Clone)]
pub struct VariableNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub node_config: VariableNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub database: DatabaseConnection,
    pub virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    variable_cache_value: Arc<RwLock<HashMap<(NodeId, i32, String), VariableValue>>>, // (node_id, config_id, variable_name) -> variable_value
}

impl VariableNodeContext {
    pub fn new(
        base_context: BacktestBaseNodeContext,
        backtest_config: VariableNodeBacktestConfig,
        heartbeat: Arc<Mutex<Heartbeat>>,
        database: DatabaseConnection,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    ) -> Self {
        Self {
            base_context,
            node_config: backtest_config,
            heartbeat,
            database,
            virtual_trading_system,
            variable_cache_value: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update_variable_cache_value(&mut self, node_id: NodeId, config_id: i32, variable_name: String, variable_value: VariableValue) {
        let mut variable_cache_value_guard = self.variable_cache_value.write().await;
        variable_cache_value_guard.insert((node_id, config_id, variable_name), variable_value);
    }

    pub async fn get_variable_cache_value(&mut self, node_id: NodeId, config_id: i32, variable_name: String) -> Option<VariableValue> {
        let variable_cache_value_guard = self.variable_cache_value.read().await;
        variable_cache_value_guard.get(&(node_id, config_id, variable_name)).cloned()
    }

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

    // pub async fn get_variable(&mut self) {
    //     let variable_configs = self.backtest_config.variable_configs.clone();

    //     for var_config in variable_configs {
    //         let variable_type = var_config.variable.clone();
    //         match variable_type {
    //             SysVariable::PositionNumber => {
    //                 let position_number = self.get_position_number(&var_config).await;
    //                 let payload = SysVariableUpdatedPayload::new(
    //                     self.get_play_index(),
    //                     var_config.config_id,
    //                     var_config.variable.clone(),
    //                     position_number as f64,
    //                 );
    //                 let sys_variable_updated_event: VariableNodeEvent = SysVariableUpdatedEvent::new(
    //                     self.base_context.node_id.clone(),
    //                     self.base_context.node_name.clone(),
    //                     var_config.output_handle_id.clone(),
    //                     payload,
    //                 )
    //                 .into();

    //                 let output_handle = self.get_output_handle(&var_config.output_handle_id);
    //                 // tracing::debug!("{}: 发送仓位数量更新事件: {:?}", self.get_node_id(), sys_variable_updated_event);
    //                 let _ = output_handle.send(sys_variable_updated_event.clone().into());

    //                 let default_output_handle = self.get_default_output_handle();
    //                 let _ = default_output_handle.send(sys_variable_updated_event.clone().into());

    //                 let strategy_output_handle = self.get_strategy_output_handle();
    //                 let _ = strategy_output_handle.send(sys_variable_updated_event.into());
    //             }
    //             _ => {}
    //         }
    //     }
    // }

    // async fn process_variable(
    //     strategy_id: i32,
    //     node_id: String,
    //     node_name: String,
    //     variables: Vec<VariableConfig>,
    //     database: DatabaseConnection,
    //     output_handle: HashMap<String, NodeOutputHandle>,
    // ) {
    //     for var in variables {
    //         let variable_type = var.variable.clone();
    //         match variable_type {
    //             SysVariable::PositionNumber => {}
    //             _ => {}
    //         }
    //     }
    // }

    // async fn get_position_number(&self, variable_config: &VariableConfig) -> u32 {
    //     let virtual_trading_system = self.virtual_trading_system.lock().await;
    //     let exchange = self
    //         .backtest_config
    //         .exchange_mode_config
    //         .as_ref()
    //         .unwrap()
    //         .selected_account
    //         .exchange
    //         .clone();
    //     let symbol = variable_config.symbol.clone();

    //     let current_positions = virtual_trading_system.get_current_positions_ref();
    //     let position_number = match symbol {
    //         // 如果symbol不为空，则获取symbol的持仓数量
    //         Some(symbol) => {
    //             let position_number = current_positions
    //                 .iter()
    //                 .filter(|position| position.symbol == symbol && position.exchange == exchange)
    //                 .count() as u32;
    //             position_number
    //         }
    //         // 如果symbol为空，则获取所有持仓数量
    //         None => {
    //             let position_number = current_positions.len() as u32;
    //             position_number
    //         }
    //     };
    //     position_number
    // }
}
