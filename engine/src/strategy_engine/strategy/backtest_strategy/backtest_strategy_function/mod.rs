mod add_edge;
mod add_node;
mod add_start_node;
mod add_kline_node;
mod add_if_else_node;
mod add_indicator_node;
mod add_order_node;
mod add_get_variable_node;
// mod add_position_node;
// pub mod sys_variable_function;
use crate::strategy_engine::node::BacktestNodeTrait;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use petgraph::{Graph, Directed};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;
use futures::stream::select_all;
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

pub struct BacktestStrategyFunction;


impl BacktestStrategyFunction {
    // 将所有节点的output_handle添加到策略中
    pub async fn add_node_output_handle(graph: &mut Graph<Box<dyn BacktestNodeTrait>, (), Directed>) -> Vec<NodeOutputHandle> {
        tracing::debug!("添加所有节点节点的输出句柄");
        let mut strategy_output_handles = Vec::new();
        // 先将所有的连接数+1
        for node in graph.node_weights_mut() {
            let output_handles = node.get_all_output_handles().await;
            for output_handle in output_handles {
                let output_handle_id = output_handle.output_handle_id.clone();
                // 增加节点的出口连接数
                node.add_output_handle_connect_count(output_handle_id).await;
            }
        }
        // 再将所有的输出句柄添加到策略中
        for node in graph.node_weights_mut() {
            let output_handles = node.get_all_output_handles().await;
            strategy_output_handles.extend(output_handles);
        }
        strategy_output_handles
    }

    pub async fn listen_node_events(context: Arc<RwLock<BacktestStrategyContext>>) {
        let (receivers, cancel_token, strategy_name) = {
            let context_guard = context.read().await;
            let receivers = context_guard.get_all_node_output_handles();
            let cancel_token = context_guard.get_cancel_token();
            let strategy_name = context_guard.get_strategy_name();
            (receivers, cancel_token, strategy_name)
        };

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", strategy_name);
            return;
        }

        // 创建一个流，用于接收节点传递过来的event
        let streams: Vec<_> = receivers.iter()
            .map(|output_handle| BroadcastStream::new(output_handle.node_event_sender.subscribe()))
            .collect();

        let mut combined_stream = select_all(streams);
        
        let context_clone = context.clone();

        

        // 节点接收数据
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点消息监听任务已中止", strategy_name);
                        break;
                    }
                    // 接收消息
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let state_guard = context_clone.write().await;
                                state_guard.handle_node_events(event).await.unwrap();
                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收消息错误: {}", strategy_name, e);
                            }
                            None => {
                                tracing::warn!("节点{}所有消息流已关闭", strategy_name);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn listen_command(context: Arc<RwLock<BacktestStrategyContext>>) {
        let (strategy_name, command_receiver) = {
            let context_guard = context.read().await;
            let strategy_name = context_guard.get_strategy_name();
            let command_receiver = context_guard.get_command_receiver();
            (strategy_name, command_receiver)

        };
        tracing::debug!("{}: 开始监听节点命令", strategy_name);
        tokio::spawn(async move {
            loop {
                // 先获取命令并立即释放锁
                let command = {
                    let received_command = command_receiver.lock().await.recv().await;
                    if let Some(cmd) = received_command {
                        cmd
                    } else {
                        continue;
                    }
                };
                // tracing::debug!("{}: 收到命令: {:?}", strategy_name, command);
                // 然后再获取context的写锁处理命令
                let mut context_guard = context.write().await;
                context_guard.handle_node_command(command).await.unwrap();
                
            }
        });
    }
}
