use star_river_core::custom_type::NodeId;
use star_river_event::backtest_strategy::node_event::{
    VariableNodeEvent,
    variable_node_event::{CustomVarUpdateEvent, CustomVarUpdatePayload},
};
use strategy_core::{
    communication::strategy::StrategyResponse,
    event::node_common_event::{CommonEvent, ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload},
    node::node_handles::NodeOutputHandle,
};
use tokio::sync::{mpsc, oneshot};

use super::VariableNodeContext;
use crate::{
    node_catalog::variable_node::context::BacktestNodeEvent,
    strategy::{
        PlayIndex,
        strategy_command::{BacktestStrategyCommand, GetCustomVarCmdPayload, GetCustomVarValueCommand},
    },
};

impl VariableNodeContext {
    /// 创建获取自定义变量的异步任务 Handle
    pub(super) async fn create_get_custom_var_handle(
        play_index: PlayIndex,
        node_id: NodeId,
        node_name: String,
        config_id: i32,
        var_name: String,
        output_handle: NodeOutputHandle<BacktestNodeEvent>,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        strategy_output_handle: NodeOutputHandle<BacktestNodeEvent>,
        is_leaf_node: bool,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let output_handle_id = output_handle.output_handle_id();

            let (resp_tx, resp_rx) = oneshot::channel();
            let get_custom_var_event = GetCustomVarCmdPayload::new(var_name.clone());
            let cmd = GetCustomVarValueCommand::new(node_id.clone(), resp_tx, get_custom_var_event);
            strategy_command_sender.send(cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();

            match response {
                StrategyResponse::Success { payload, .. } => {
                    let var_op = "get".to_string();
                    let custom_variable = payload.custom_variable;
                    let payload = CustomVarUpdatePayload::new(play_index, config_id, var_op, None, None, custom_variable.clone());
                    let var_event: VariableNodeEvent =
                        CustomVarUpdateEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into();
                    let _ = strategy_output_handle.send(var_event.clone().into());
                    if is_leaf_node {
                        let payload = ExecuteOverPayload::new(play_index as u64, Some(config_id));
                        let execute_over_event: CommonEvent =
                            ExecuteOverEvent::new(node_id, node_name, output_handle_id.clone(), payload).into();
                        let _ = strategy_output_handle.send(execute_over_event.into());
                    } else {
                        let _ = output_handle.send(var_event.clone().into());
                    }
                }
                StrategyResponse::Fail { error, .. } => {
                    tracing::error!("get_variable failed: {:?}", error);
                    let payload = TriggerPayload::new(play_index as u64);
                    let trigger_event: CommonEvent = TriggerEvent::new(node_id, node_name, output_handle_id.clone(), payload).into();
                    let backtest_trigger_event: BacktestNodeEvent = trigger_event.into();
                    let _ = output_handle.send(backtest_trigger_event);
                }
            }
        })
    }
}
