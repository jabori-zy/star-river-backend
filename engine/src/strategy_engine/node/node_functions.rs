use super::node_context::{BacktestNodeContextTrait, LiveNodeContextTrait};
use crate::{strategy_engine::node::node_types::NodeType, EngineName};
use event_center::event::Event;
use event_center::Channel;
use event_center::EventCenterSingleton;
use futures::stream::select_all;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;

pub struct LiveNodeFunction;

impl LiveNodeFunction {
    pub async fn listen_external_event(context: Arc<RwLock<Box<dyn LiveNodeContextTrait>>>) {
        let (event_receivers, cancel_token, node_id) = {
            // let state_guard = state.read().await;
            // 这里需要深度克隆接收器，而不是克隆引用
            let event_receivers: Vec<broadcast::Receiver<Event>> = context
                .read()
                .await
                .get_event_receivers()
                .iter()
                .map(|r| r.resubscribe())
                .collect();

            let cancel_token = context.read().await.get_cancel_token().clone();
            let node_id = context.read().await.get_node_id().to_string();
            (event_receivers, cancel_token, node_id)
        };

        if event_receivers.is_empty() {
            tracing::warn!("{}: 没有事件接收器", node_id);
            return;
        }
        let streams: Vec<_> = event_receivers
            .into_iter()
            .map(|receiver| BroadcastStream::new(receiver))
            .collect();
        let mut combined_stream = select_all(streams);
        let node_id = node_id.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点监听外部事件进程已中止", node_id);
                        break;
                    }
                    // 接收消息
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                context_guard.handle_event(event).await;

                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收事件错误: {}", node_id, e);
                            }
                            None => {
                                tracing::warn!("节点{}所有事件流已关闭", node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn listen_message(context: Arc<RwLock<Box<dyn LiveNodeContextTrait>>>) {
        let (receivers, cancel_token, node_id) = {
            let state_guard = context.read().await;
            let receivers = state_guard.get_message_receivers().clone();
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            (receivers, cancel_token, node_id)
        };

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", node_id);
            return;
        }

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = receivers
            .iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();

        let mut combined_stream = select_all(streams);
        let state = context.clone();

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
                                // tracing::debug!("{} 收到消息: {:?}", node_id, message);
                                let mut state_guard = state.write().await;
                                state_guard.handle_message(message).await;
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
    pub async fn cancel_task(state: Arc<RwLock<Box<dyn LiveNodeContextTrait>>>) {
        let (cancel_token, node_id, run_state) = {
            let state_guard = state.read().await;
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            let run_state = state_guard.get_run_state();
            (cancel_token, node_id, run_state)
        };

        cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", node_id, run_state);
    }
}

static BACKTEST_NODE_EVENT_RECEIVERS: LazyLock<HashMap<NodeType, Vec<Channel>>> =
    LazyLock::new(|| {
        HashMap::from([
            (NodeType::StartNode, vec![]),
            (NodeType::KlineNode, vec![Channel::Market]),
            (NodeType::IndicatorNode, vec![]),
            (NodeType::IfElseNode, vec![]),
            (NodeType::FuturesOrderNode, vec![]),
            (NodeType::PositionNode, vec![]),
            (NodeType::PositionManagementNode, vec![]),
            (NodeType::GetVariableNode, vec![]),
            (NodeType::OrderNode, vec![]),
            (NodeType::VariableNode, vec![]),
        ])
    });

pub struct BacktestNodeEventReceiver;

impl BacktestNodeEventReceiver {
    pub fn get_backtest_node_event_receivers(node_type: &NodeType) -> Vec<Channel> {
        BACKTEST_NODE_EVENT_RECEIVERS
            .get(node_type)
            .cloned()
            .unwrap_or_default()
    }
}

pub struct BacktestNodeFunction;

impl BacktestNodeFunction {
    pub async fn listen_external_event(context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>) {
        let (event_receivers, cancel_token, node_id) = {
            // let state_guard = state.read().await;
            // 这里需要深度克隆接收器，而不是克隆引用
            // let event_receivers : Vec<broadcast::Receiver<Event>> = context.read().await.get_event_receivers()
            // .iter()
            // .map(|r| r.resubscribe())
            // .collect();
            let context_guard = context.read().await;

            let cancel_token = context_guard.get_cancel_token().clone();
            let node_id = context_guard.get_node_id().to_string();
            let node_type = context_guard.get_node_type();
            let should_receive_channels =
                BacktestNodeEventReceiver::get_backtest_node_event_receivers(node_type);

            let mut event_receivers = Vec::new();
            for channel in should_receive_channels.iter() {
                let event_receiver = EventCenterSingleton::subscribe(channel).await.unwrap();
                event_receivers.push(event_receiver);
            }

            (event_receivers, cancel_token, node_id)
        };

        if event_receivers.is_empty() {
            tracing::warn!("{}: 没有事件接收器", node_id);
            return;
        }
        let streams: Vec<_> = event_receivers
            .into_iter()
            .map(|receiver| BroadcastStream::new(receiver))
            .collect();
        let mut combined_stream = select_all(streams);
        let node_id = node_id.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 外部事件监听任务已中止", node_id);
                        break;
                    }
                    // 接收消息
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                context_guard.handle_event(event).await;

                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收事件错误: {}", node_id, e);
                            }
                            None => {
                                tracing::warn!("节点{}所有事件流已关闭", node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn listen_node_events(context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>) {
        let (input_handles, cancel_token, node_id) = {
            let state_guard = context.read().await;
            let input_handles = state_guard.get_all_input_handles().clone();
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            (input_handles, cancel_token, node_id)
        };

        if input_handles.is_empty() {
            tracing::warn!("{}: 没有消息接收器", node_id);
            return;
        }

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = input_handles
            .iter()
            .map(|input_handle| BroadcastStream::new(input_handle.get_receiver()))
            .collect();

        let mut combined_stream = select_all(streams);
        let state = context.clone();

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
                                // tracing::debug!("{} 收到消息: {:?}", node_id, message);
                                let mut state_guard = state.write().await;
                                state_guard.handle_node_event(message).await;
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

    pub async fn listen_strategy_inner_events(
        context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
    ) {
        let (inner_event_receiver, cancel_token, node_id) = {
            let state_guard = context.read().await;
            let receiver = state_guard
                .get_strategy_inner_event_receiver()
                .resubscribe();
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            (receiver, cancel_token, node_id)
        };

        // 创建一个流，用于接收节点传递过来的message
        let mut stream = BroadcastStream::new(inner_event_receiver);
        let state = context.clone();

        // 节点接收数据
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 策略内部事件监听任务已中止", node_id);
                        break;
                    }
                    // 接收消息
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(message)) => {
                                // tracing::debug!("{} 收到消息: {:?}", node_id, message);
                                let mut state_guard = state.write().await;
                                state_guard.handle_strategy_inner_event(message).await;
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

    pub async fn listen_strategy_command(context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>) {
        let (strategy_command_receiver, cancel_token, node_id) = {
            let state_guard = context.read().await;
            let receiver = state_guard.get_strategy_command_receiver();
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            (receiver, cancel_token, node_id)
        };

        // 节点接收数据
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 策略命令监听任务已中止", node_id);
                        break;
                    }

                    _ = async {
                        if let Some(received_command) = strategy_command_receiver.lock().await.recv().await {
                            let mut context_guard = context.write().await;
                            context_guard.handle_strategy_command(received_command).await;
                        }
                    } => {}
                }
            }
        });
    }

    // pub async fn listen_play_index(context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>) {
    //     let (mut play_index_watch_rx, cancel_token, node_id) = {
    //         let state_guard = context.read().await;
    //         let play_index_watch_rx = state_guard.get_play_index_watch_rx();
    //         let cancel_token = state_guard.get_cancel_token().clone();
    //         let node_id = state_guard.get_node_id().to_string();
    //         (play_index_watch_rx, cancel_token, node_id)
    //     };

    //     // 节点接收播放索引变化
    //   tokio::spawn(async move {
    //         loop {
    //             tokio::select! {
    //                 // 如果取消信号被触发，则中止任务
    //                 _ = cancel_token.cancelled() => {
    //                     tracing::info!("{} 播放索引监听任务已中止", node_id);
    //                     break;
    //                 }
    //                 // 监听播放索引变化
    //                 receive_result = play_index_watch_rx.changed() => {
    //                     match receive_result {
    //                         Ok(_) => {
    //                             // 获取最新的播放索引
    //                             let play_index = *play_index_watch_rx.borrow_and_update();

    //                             let mut state_guard = context.write().await;
    //                             state_guard.handle_play_index(play_index).await.unwrap();
    //                         }
    //                         Err(e) => {
    //                             tracing::error!("节点{}监听播放索引错误: {}", node_id, e);
    //                             break;
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     });
    // }

    /// 通用的任务取消实现
    pub async fn cancel_task(context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>) {
        let state_guard = context.read().await;
        state_guard.get_cancel_token().cancel();
    }
}
