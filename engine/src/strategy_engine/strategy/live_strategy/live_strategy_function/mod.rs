mod add_edge;
mod add_node;
mod add_start_node;
mod add_kline_node;
mod add_if_else_node;
mod add_indicator_node;
mod add_order_node;
mod add_position_node;
mod add_get_variable_node;
pub mod sys_variable_function;
use crate::strategy_engine::node::LiveNodeTrait;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use petgraph::{Graph, Directed};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use event_center::Event;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::strategy::live_strategy::live_strategy_context::LiveStrategyContext;
use crate::strategy_engine::node::node_state_machine::LiveNodeRunState;


pub struct LiveStrategyFunction;


impl LiveStrategyFunction {
    // 将所有节点的output_handle添加到策略中
    pub async fn add_node_output_handle(graph: &mut Graph<Box<dyn LiveNodeTrait>, (), Directed>) -> Vec<NodeOutputHandle> {
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

    pub async fn listen_event(context: Arc<RwLock<LiveStrategyContext>>){
        let (event_receivers, cancel_token, strategy_name) = {
            // let state_guard = state.read().await;
            // 这里需要深度克隆接收器，而不是克隆引用
            let event_receivers : Vec<broadcast::Receiver<Event>> = context.read().await.get_event_receivers()
            .iter()
            .map(|r| r.resubscribe())
            .collect();

            let cancel_token = context.read().await.get_cancel_token();
            let strategy_name = context.read().await.get_strategy_name().to_string();
            (event_receivers, cancel_token, strategy_name)
        };

        if event_receivers.is_empty() {
            tracing::warn!("{}: 没有事件接收器", strategy_name);
            return;
        }
        let streams: Vec<_> = event_receivers.into_iter()
            .map(|receiver| BroadcastStream::new(receiver))
            .collect();
        let mut combined_stream = select_all(streams);
        let strategy_name = strategy_name.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 策略监听外部事件进程已中止", strategy_name);
                        break;
                    }
                    // 接收消息
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                context_guard.handle_event(event).await.unwrap();

                            }
                            Some(Err(e)) => {
                                tracing::error!("策略{}接收事件错误: {}", strategy_name, e);
                            }
                            None => {
                                tracing::warn!("策略{}所有事件流已关闭", strategy_name);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn listen_node_message(context: Arc<RwLock<LiveStrategyContext>>) {
        let (receivers, cancel_token, strategy_name) = {
            let context_guard = context.read().await;
            let receivers = context_guard.get_all_node_output_handles();
            let cancel_token = context_guard.get_cancel_token();
            let strategy_name = context_guard.get_strategy_name().to_string();
            (receivers, cancel_token, strategy_name)
        };

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", strategy_name);
            return;
        }

        // 创建一个流，用于接收节点传递过来的message
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
                            Some(Ok(message)) => {
                                // tracing::debug!("{} 收到消息: {:?}", node_id, message);
                                let mut state_guard = context_clone.write().await;
                                state_guard.handle_node_message(message).await.unwrap();
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

    pub async fn listen_command(context: Arc<RwLock<LiveStrategyContext>>) {
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
                tracing::debug!("{}: 收到命令: {:?}", strategy_name, command);
                // 然后再获取context的写锁处理命令
                let context_guard = context.try_write();
                if let Ok(mut context_guard) = context_guard {
                    context_guard.handle_command(command).await.unwrap();
                } else {
                    tracing::error!("{}: 获取context写锁失败", strategy_name);
                }
            }
        });
    }

    pub async fn start_node(node: &Box<dyn LiveNodeTrait>) -> Result<(), String> {
        // 启动节点
        let mut node_clone = node.clone();
        
        let node_handle = tokio::spawn(async move {
            let node_name = node_clone.get_node_name().await;
            if let Err(e) = node_clone.start().await {
                tracing::error!("{} 节点启动失败: {}", node_name, e);
                return Err(format!("节点启动失败: {}", e));
            }
            Ok(())
        });

        let node_name = node.get_node_name().await;
        let node_id = node.get_node_id().await;
        
        
        // 等待节点启动完成
        match tokio::time::timeout(Duration::from_secs(30), node_handle).await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("节点 {} 启动任务失败: {}", node_name, e));
                }
                
                if let Ok(Err(e)) = result {
                    return Err(format!("节点 {} 启动过程中出错: {}", node_name, e));
                }
            }
            Err(_) => {
                return Err(format!("节点 {} 启动超时", node_id));
            }
        }
        
        // 等待节点进入Running状态
        let mut retry_count = 0;
        let max_retries = 50;
        
        while retry_count < max_retries {
            let run_state = node.get_run_state().await;
            if run_state == LiveNodeRunState::Running {
                tracing::debug!("节点 {} 已进入Running状态", node_id);
                // 节点启动间隔
                // tokio::time::sleep(Duration::from_millis(1000)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(format!("节点 {} 未能进入Running状态", node_id))

    }

    pub async fn cancel_task(context: Arc<RwLock<LiveStrategyContext>>) {
        let (cancel_token, strategy_name, run_state) = {
            let state_guard = context.read().await;
            let cancel_token: CancellationToken = state_guard.get_cancel_token().clone();
            let strategy_name = state_guard.get_strategy_name().to_string();
            let run_state = state_guard.get_state_machine().current_state();
            (cancel_token, strategy_name, run_state)
        };
        
        cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", strategy_name, run_state);
    }
}
