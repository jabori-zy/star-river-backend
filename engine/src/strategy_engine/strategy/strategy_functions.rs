use std::sync::Arc;
use tokio::sync::RwLock;
use event_center::Event;
use tokio::sync::broadcast;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;
// use super::node::node_types::NodeContext;
pub struct StrategyFunction;
use types::strategy::message::NodeMessage;
use tokio_util::sync::CancellationToken;
use crate::strategy_engine::strategy::strategy_context::StrategyContext;

impl StrategyFunction {
    // pub async fn listen_external_event(context: Arc<RwLock<Box<dyn NodeContext>>>){
    //     let (event_receivers, cancel_token, node_id) = {
    //         // let state_guard = state.read().await;
    //         // 这里需要深度克隆接收器，而不是克隆引用
    //         let event_receivers : Vec<broadcast::Receiver<Event>> = context.read().await.get_event_receivers()
    //         .iter()
    //         .map(|r| r.resubscribe())
    //         .collect();

    //         let cancel_token = context.read().await.get_cancel_token().clone();
    //         let node_id = context.read().await.get_node_id().to_string();
    //         (event_receivers, cancel_token, node_id)
    //     };

    //     if event_receivers.is_empty() {
    //         tracing::warn!("{}: 没有事件接收器", node_id);
    //         return;
    //     }
    //     let streams: Vec<_> = event_receivers.into_iter()
    //         .map(|receiver| BroadcastStream::new(receiver))
    //         .collect();
    //     let mut combined_stream = select_all(streams);
    //     let node_id = node_id.clone();
    //     tokio::spawn(async move {
    //         loop {
    //             tokio::select! {
    //                 _ = cancel_token.cancelled() => {
    //                     tracing::info!("{} 节点监听外部事件进程已中止", node_id);
    //                     break;
    //                 }
    //                 // 接收消息
    //                 receive_result = combined_stream.next() => {
    //                     match receive_result {
    //                         Some(Ok(event)) => {
    //                             let mut context_guard = context.write().await;
    //                             context_guard.handle_event(event).await.unwrap();

    //                         }
    //                         Some(Err(e)) => {
    //                             tracing::error!("节点{}接收事件错误: {}", node_id, e);
    //                         }
    //                         None => {
    //                             tracing::warn!("节点{}所有事件流已关闭", node_id);
    //                             break;
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     });
    // }

    pub async fn listen_node_message(context: Arc<RwLock<Box<dyn StrategyContext>>>) {
        let (receivers, cancel_token, strategy_name) = {
            let context_guard = context.read().await;
            let receivers = context_guard.get_all_node_output_handles();
            let cancel_token = context_guard.get_cancel_token().clone();
            let strategy_name = context_guard.get_strategy_name().to_string();
            (receivers, cancel_token, strategy_name)
        };

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", strategy_name);
            return;
        }

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = receivers.iter()
            .map(|output_handle| BroadcastStream::new(output_handle.message_sender.subscribe()))
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

    /// 通用的任务取消实现
    pub async fn cancel_task(context: Arc<RwLock<Box<dyn StrategyContext>>>) 
    {
        let (cancel_token, strategy_name, run_state) = {
            let state_guard = context.read().await;
            let cancel_token = state_guard.get_cancel_token().clone();
            let strategy_name = state_guard.get_strategy_name().to_string();
            let run_state = state_guard.get_run_state();
            (cancel_token, strategy_name, run_state)
        };
        
        cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", strategy_name, run_state);
    }
    
}