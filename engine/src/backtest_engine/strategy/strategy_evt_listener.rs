use super::BacktestStrategy;
use futures::StreamExt;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;

impl BacktestStrategy {
    pub async fn listen_node_events(&self) {
        let (receivers, cancel_token, strategy_name) = self
            .with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    let mut nodes = ctx.topological_sort().unwrap();
                    let mut receivers = Vec::new();
                    let strategy_name = ctx.strategy_name();
                    for node in nodes.iter_mut() {
                        let receiver = node.subscribe_strategy_output_handle(strategy_name.clone()).await;
                        receivers.push(receiver);
                    }

                    let cancel_token = ctx.cancel_task_token();
                    (receivers, cancel_token, strategy_name.clone())
                })
            })
            .await;

        if receivers.is_empty() {
            tracing::warn!("{}: 没有消息接收器", strategy_name);
            return;
        }

        // 创建一个流，用于接收节点传递过来的event
        let streams: Vec<_> = receivers.into_iter().map(|receiver| BroadcastStream::new(receiver)).collect();

        let mut combined_stream = select_all(streams);

        let context_clone = self.context.clone();

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

    pub async fn listen_strategy_command(&self) {
        let (strategy_name, command_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let strategy_name = ctx.strategy_name();
                    let command_receiver = ctx.strategy_command_receiver();
                    (strategy_name.clone(), command_receiver)
                })
            })
            .await;

        tracing::debug!("strategy command receiver: {:?}", command_receiver.lock().await);
        let context = self.context.clone();
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
                // 然后再获取context的写锁处理命令
                let mut context_guard = context.write().await;
                context_guard.handle_strategy_command(command).await.unwrap();
            }
        });
    }

    pub async fn listen_strategy_stats_event(&self) {
        let (strategy_name, cancel_token, strategy_stats_event_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let strategy_name = ctx.strategy_name();
                    let cancel_token = ctx.cancel_task_token();
                    let strategy_stats_event_receiver = ctx.strategy_stats_event_receiver();
                    (strategy_name.clone(), cancel_token, strategy_stats_event_receiver)
                })
            })
            .await;

        let mut stream = BroadcastStream::new(strategy_stats_event_receiver);

        let context = self.context.clone();
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
}
