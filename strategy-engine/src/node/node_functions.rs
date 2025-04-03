use crate::{NodeMessageReceiver,NodeRunState};
use types::strategy::message::NodeMessage;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use tokio::sync::RwLock;
use event_center::Event;
use tokio::sync::broadcast;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;


pub struct NodeFunction;


impl NodeFunction {
    pub async fn listen_external_event<T: Send + Sync + 'static>(
        state: Arc<RwLock<T>>,
        get_receiver: impl Fn(&T) -> &broadcast::Receiver<Event>,
        get_cancel_token: impl Fn(&T) -> &CancellationToken,
        get_node_id: impl Fn(&T) -> &str,
        internal_tx: tokio::sync::mpsc::Sender<Event>,
    ){
        let (mut receiver, cancel_token, node_id) = {
            let state_guard = state.read().await;
            let receiver = get_receiver(&state_guard).resubscribe();
            let cancel_token = get_cancel_token(&state_guard).clone();
            let node_id = get_node_id(&state_guard).to_string();
            (receiver, cancel_token, node_id)
        };

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点监听外部事件进程已中止", node_id);
                        break;
                    }
                    Ok(event) = receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
    }

    pub async fn listen_message<T: Send + Sync + 'static>(
        state: Arc<RwLock<T>>,
        get_receiver: impl Fn(&T) -> &Vec<NodeMessageReceiver>,
        get_cancel_token: impl Fn(&T) -> &CancellationToken,
        get_node_id: impl Fn(&T) -> &str,
        process_message: impl Fn(NodeMessage, Arc<RwLock<T>>) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static,
    ) {
        let (receivers, cancel_token, node_id) = {
            let state_guard = state.read().await;
            let receivers = get_receiver(&state_guard).clone();
            let cancel_token = get_cancel_token(&state_guard).clone();
            let node_id = get_node_id(&state_guard).to_string();
            (receivers, cancel_token, node_id)
        };

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", node_id);
            return;
        }

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();

        let mut combined_stream = select_all(streams);
        let state = state.clone();

        

        // 节点接收数据
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点消息监听任务已中止", node_id);
                        break;
                    }
                    // 接收消息
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(message)) => {
                                process_message(message, state.clone()).await;
                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收消息错误: {}", node_id, e);
                            }
                            None => {
                                tracing::warn!("节点{}所有消息流已关闭", node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    /// 通用的任务取消实现
    pub async fn cancel_task<T: Send + Sync + 'static>(
        state: &Arc<RwLock<T>>,
        get_cancel_token: impl Fn(&T) -> &CancellationToken,
        get_node_id: impl Fn(&T) -> &str,
        get_run_state: impl Fn(&T) -> NodeRunState,
    ) {
        let (cancel_token, node_id, run_state) = {
            let state_guard = state.read().await;
            let cancel_token = get_cancel_token(&*state_guard).clone();
            let node_id = get_node_id(&*state_guard).to_string();
            let run_state = get_run_state(&*state_guard);
            (cancel_token, node_id, run_state)
        };
        
        cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", node_id, run_state);
    }
    
}