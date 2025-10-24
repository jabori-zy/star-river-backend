use super::{
    NodeOutputHandle, VariableNodeContext,
    BacktestStrategyCommand, GetCustomVariableCmdPayload, GetCustomVariableValueCommand, Response,
    ExecuteOverEvent, ExecuteOverPayload, TriggerEvent, TriggerPayload,
    CustomVariableUpdateEvent, CustomVariableUpdatePayload,
    BacktestNodeEvent, CommonEvent, VariableNodeEvent,
    NodeId, PlayIndex,
};
use tokio::sync::{mpsc, oneshot};

impl VariableNodeContext {
    /// 创建获取自定义变量的异步任务 Handle
    pub(super) async fn create_get_custom_var_handle(
        play_index: PlayIndex,
        node_id: NodeId,
        node_name: String,
        config_id: i32,
        var_name: String,
        var_display_name: String,
        output_handle: NodeOutputHandle,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        strategy_output_handle: NodeOutputHandle,
        is_leaf_node: bool,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {


            let output_handle_id = output_handle.output_handle_id();

            let (resp_tx, resp_rx) = oneshot::channel();
            let get_custom_var_event = GetCustomVariableCmdPayload::new(var_name.clone());
            let cmd = GetCustomVariableValueCommand::new(node_id.clone(), resp_tx, Some(get_custom_var_event));
            strategy_command_sender.send(cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();

            if response.is_success() {
                let var_op = "get".to_string();
                let payload = CustomVariableUpdatePayload::new(
                    play_index,
                    config_id,
                    var_op,
                    None,
                    None,
                    response.custom_variable.clone(),
                );
                let var_event: VariableNodeEvent =
                    CustomVariableUpdateEvent::new(node_id.clone(), node_name.clone(), output_handle_id.clone(), payload).into();

                // 转换为 BacktestNodeEvent
                let backtest_var_event: BacktestNodeEvent = var_event.clone().into();

                // 发送到策略
                let _ = strategy_output_handle.send(backtest_var_event.clone());

                // 如果是叶子节点，则发送执行结束事件
                if is_leaf_node {
                    let payload = ExecuteOverPayload::new(play_index);
                    let execute_over_event: CommonEvent =
                        ExecuteOverEvent::new(node_id, node_name, output_handle_id.clone(), payload).into();
                    let backtest_execute_over_event: BacktestNodeEvent = execute_over_event.into();
                    let _ = strategy_output_handle.send(backtest_execute_over_event);
                } else {
                    let _ = output_handle.send(backtest_var_event);
                }
            } else {
                tracing::error!("get_variable failed: {:?}", response.get_error());
                // 失败，发送触发事件
                let payload = TriggerPayload::new(play_index);
                let trigger_event: CommonEvent = TriggerEvent::new(node_id, node_name, output_handle_id, payload).into();
                let backtest_trigger_event: BacktestNodeEvent = trigger_event.into();
                let _ = output_handle.send(backtest_trigger_event);
            }
        })
    }
}

