mod context_impl;
mod context_utils;
mod variable_handler;
mod sys_variable_handler;
mod custom_variable_handler;

use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_types::NodeOutputHandle;
use event_center::communication::Command;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::*;
use event_center::event::Event;
use event_center::event::node_event::NodeEventTrait;
use event_center::event::node_event::backtest_node_event::if_else_node_event::ConditionMatchEvent;
use event_center::event::node_event::backtest_node_event::{
    BacktestNodeEvent, IfElseNodeEvent, CommonEvent,
    variable_node_event::{
        SysVariableUpdatePayload, SysVariableUpdateEvent, VariableNodeEvent,
        CustomVariableUpdateEvent, CustomVariableUpdatePayload,
    },
    common_event::{ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
};
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::{NodeId, PlayIndex};
use star_river_core::error::engine_error::node_error::variable_node::SysVariableSymbolIsNullSnafu;
use star_river_core::error::engine_error::node_error::VariableNodeError;
use star_river_core::market::QuantData;
use star_river_core::node::variable_node::trigger::dataflow::{DataFlow, DataflowErrorPolicy, DataflowErrorType};
use star_river_core::node::variable_node::trigger::{ConditionTrigger, TriggerConfig};
use star_river_core::node::variable_node::variable_config::{GetVariableConfig, ResetVariableConfig, UpdateVariableConfig};
use star_river_core::node::variable_node::variable_config::get::GetSystemVariableConfig;
use star_river_core::node::variable_node::*;
use star_river_core::order::OrderStatus;
use star_river_core::strategy::{
    custom_variable::VariableValue, 
    sys_varibale::{SysVariable, SysVariableType},
};
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

    pub async fn update_variable_cache_value(
        &mut self,
        node_id: NodeId,
        config_id: i32,
        variable_name: String,
        variable_value: VariableValue,
    ) {
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
}
