mod add_edge;
mod add_futures_order_node;
mod add_variable_node;
mod add_if_else_node;
mod add_indicator_node;
mod add_kline_node;
mod add_node;
mod add_position_node;
mod add_start_node;

use super::{BacktestNodeTrait,BacktestStrategyContext, BacktestNodeContextTrait};
use super::context::{
    IndicatorNodeContext, 
    KlineNodeContext, 
    FuturesOrderNodeContext, 
    IfElseNodeContext, 
    StartNodeContext, 
    VariableNodeContext,
    PositionNodeContext
};
use super::node_handles::NodeType;
use super::node::{FuturesOrderNode, IfElseNode, IndicatorNode, KlineNode, PositionManagementNode, StartNode, VariableNode};
use super::node_handles::NodeInputHandle;
use futures::StreamExt;
use futures::stream::select_all;
use petgraph::{Direction, graph::NodeIndex};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;
use super::node::BacktestNodeContextAccessor;
use star_river_core::error::engine_error::node_error::BacktestStrategyNodeError;


pub struct BacktestStrategyFunction;

impl BacktestStrategyFunction {

    pub async fn listen_node_events(context: Arc<RwLock<BacktestStrategyContext>>) {

        let (receivers, cancel_token, strategy_name) = {
            let context_guard = context.write().await;
            let all_node = context_guard.topological_sort();
            let mut receivers = Vec::new();
            let strategy_name = context_guard.get_strategy_name();
            
            for node in all_node {
                let receiver = node.with_ctx_write_dyn(|ctx| {
                    ctx.get_strategy_output_handle_mut().subscribe(strategy_name.clone())
                }).await;
                receivers.push(receiver);
            }

            let cancel_token = context_guard.get_cancel_task_token();
            (receivers, cancel_token, strategy_name)
        };

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", strategy_name);
            return;
        }

        // 创建一个流，用于接收节点传递过来的event
        let streams: Vec<_> = receivers
            .into_iter()
            .map(|receiver| BroadcastStream::new(receiver))
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
                                let mut state_guard = context_clone.write().await;
                                state_guard.handle_node_event(event).await;
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

    pub async fn listen_strategy_command(context: Arc<RwLock<BacktestStrategyContext>>) {
        let (strategy_name, command_receiver) = {
            let context_guard = context.read().await;
            let strategy_name = context_guard.get_strategy_name();
            let command_receiver = context_guard.get_strategy_command_receiver();
            (strategy_name, command_receiver)
        };
        tracing::debug!("{}: 开始监听节点命令", strategy_name);
        tokio::spawn(async move {
            loop {
                // 先获取命令并立即释放锁
                let command = {
                    let mut command_receiver_guard = command_receiver.lock().await;
                    let received_command = command_receiver_guard.recv().await;
                    if let Some(cmd) = received_command {
                        cmd
                    } else {
                        continue;
                    }
                };
                // tracing::debug!("{}: 收到命令: {:?}", strategy_name, command);
                // 然后再获取context的写锁处理命令
                let mut context_guard = context.write().await;
                context_guard.handle_strategy_command(command).await.unwrap();
            }
        });
    }

    pub async fn listen_strategy_stats_event(context: Arc<RwLock<BacktestStrategyContext>>) {
        let (strategy_name, cancel_token, strategy_stats_event_receiver) = {
            let context_guard = context.read().await;
            let strategy_name = context_guard.get_strategy_name();
            let cancel_token = context_guard.get_cancel_task_token();
            let strategy_stats_event_receiver = context_guard.strategy_stats_event_receiver.resubscribe();
            (strategy_name, cancel_token, strategy_stats_event_receiver)
        };

        let mut stream = BroadcastStream::new(strategy_stats_event_receiver);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{}: 策略统计事件监听任务已中止", strategy_name);
                        break;
                    }
                    event = stream.next() => {
                        match event {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                context_guard.handle_strategy_stats_event(event).await.unwrap();
                            }
                        Some(Err(e)) => {
                            tracing::error!("{}: 策略统计事件接收错误: {}", strategy_name, e);
                        }
                        None => {
                            tracing::warn!("{}: 策略统计事件流已关闭", strategy_name);
                            break;
                        }
                        }
                    }
                }
            }
        });
    }

    pub async fn set_leaf_nodes(
        context: Arc<RwLock<BacktestStrategyContext>>,
    ) -> Result<(), BacktestStrategyNodeError> {
        let mut context_guard = context.write().await;
        let leaf_nodes: Vec<NodeIndex> = context_guard.graph.externals(Direction::Outgoing).collect();
        let mut leaf_node_ids = Vec::new();
        for node_index in leaf_nodes {
            if let Some(node) = context_guard.graph.node_weight_mut(node_index) {
                let node_id = node
                    .with_ctx_write_dyn(|ctx| {
                        ctx.set_is_leaf_node(true);
                        ctx.get_node_id().clone()
                    })
                    .await;
                leaf_node_ids.push(node_id);
            }
        }
        context_guard.set_leaf_node_ids(leaf_node_ids);
        Ok(())
    }
}
