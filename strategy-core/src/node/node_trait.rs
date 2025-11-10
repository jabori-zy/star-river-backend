// std
use std::{future::Future, pin::Pin, sync::Arc};

// third-party
use async_trait::async_trait;
use star_river_core::{custom_type::NodeId, error::StarRiverErrorTrait};
use tokio::sync::RwLock;

// workspace crate

// current crate
use super::{
    context_trait::{
        NodeCommunicationExt, NodeContextExt, NodeEventHandlerExt, NodeHandleExt, NodeIdentityExt, NodeMetaDataExt, NodeTaskControlExt,
    },
    node_state_machine::StateTransTrigger,
};
use crate::error::NodeStateMachineError;

#[async_trait]
pub trait NodeTrait: Clone + Send + Sync + 'static {
    async fn node_id(&self) -> NodeId;
}

// ============================================================================
// 节点上下文访问器（提供便捷的读写锁访问方法）
// ============================================================================

/// 节点上下文访问器
///
/// 为泛型节点提供便捷的上下文访问方法，
/// 封装了读锁/写锁的获取和释放逻辑，避免样板代码
#[async_trait]
pub trait NodeContextAccessor: Send + Sync {
    /// 上下文类型，必须实现 NodeMetaDataTrait
    type Context: NodeMetaDataExt;

    /// 获取上下文的共享引用
    fn context(&self) -> &Arc<RwLock<Self::Context>>;

    /// 以读锁方式访问上下文（同步闭包）
    ///
    /// # 示例
    /// ```rust
    /// let node_name = node.with_ctx_read(|ctx| {
    ///     ctx.node_name().to_string()
    /// }).await;
    /// ```
    async fn with_ctx_read<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a Self::Context) -> R + Send,
        R: Send,
    {
        let guard = self.context().read().await;
        f(&*guard)
    }

    /// 以写锁方式访问上下文（同步闭包）
    ///
    /// # 示例
    /// ```rust
    /// node.with_ctx_write(|ctx| {
    ///     ctx.set_leaf_node(true);
    /// }).await;
    /// ```
    async fn with_ctx_write<R, F>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(&'a mut Self::Context) -> R + Send,
        R: Send,
    {
        let mut guard = self.context().write().await;
        f(&mut *guard)
    }

    /// 以读锁方式访问上下文（异步闭包）
    ///
    /// # 示例
    /// ```rust
    /// node.with_ctx_read_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.send_execute_over_event().await
    ///     })
    /// }).await;
    /// ```
    async fn with_ctx_read_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a Self::Context) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let guard = self.context().read().await;
        f(&*guard).await
    }

    /// 以写锁方式访问上下文（异步闭包）
    ///
    /// # 示例
    /// ```rust
    /// node.with_ctx_write_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.handle_node_command(cmd).await;
    ///     })
    /// }).await;
    /// ```
    async fn with_ctx_write_async<R>(
        &self,
        f: impl for<'a> FnOnce(&'a mut Self::Context) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send,
    ) -> R
    where
        R: Send,
    {
        let mut guard = self.context().write().await;
        f(&mut *guard).await
    }
}

// ============================================================================
// 节点生命周期管理
// ============================================================================

/// 节点生命周期管理 trait
///
/// 定义节点的生命周期相关操作（初始化、停止、状态更新）
/// 依赖 `NodeContextAccessor` 来访问上下文
#[async_trait]
pub trait NodeLifecycle: NodeContextAccessor {
    /// 错误类型
    type Error: StarRiverErrorTrait;

    /// 状态转换触发器类型
    type Trigger: StateTransTrigger;

    /// 初始化节点
    ///
    /// 在节点开始运行前调用，用于执行必要的初始化操作
    /// 具体节点可以重写此方法来实现自定义初始化逻辑
    async fn init(&self) -> Result<(), Self::Error>;

    /// 停止节点
    ///
    /// 优雅地停止节点，清理资源
    /// 具体节点可以重写此方法来实现自定义清理逻辑
    async fn stop(&self) -> Result<(), Self::Error>;

    /// 更新节点状态
    ///
    /// 处理节点状态转换事件
    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error>;
}

// ============================================================================
// 节点事件监听
// ============================================================================

/// 节点事件监听 trait
///
/// 定义节点监听各种事件的方法（外部事件、上游节点事件、策略命令）
/// 依赖 `NodeContextAccessor` 来访问上下文
#[async_trait]
pub trait NodeEventListener: NodeContextAccessor
where
    Self::Context: NodeContextExt,
{
    /// 监听外部事件（引擎事件）
    ///
    /// 根据节点类型订阅相应的事件通道，并在后台任务中处理接收到的事件
    async fn listen_engine_event(&self) {
        // TODO: 实现引擎事件监听逻辑
        // 需要根据具体的引擎事件系统实现订阅逻辑
        tracing::warn!("listen_engine_event 需要具体实现");
    }

    /// 监听节点事件（来自上游节点的消息）
    ///
    /// 监听输入句柄接收到的节点消息
    async fn listen_source_node_events(&self) {
        use futures::{StreamExt, stream::select_all};
        use tokio_stream::wrappers::BroadcastStream;

        let (input_handles, cancel_token, node_id) = self
            .with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    let input_handles = ctx.input_handles().to_vec();
                    let cancel_token = ctx.cancel_token().clone();
                    let node_id = ctx.node_id().to_string();
                    (input_handles, cancel_token, node_id)
                })
            })
            .await;

        if input_handles.is_empty() {
            tracing::warn!("{}: 没有消息接收器", node_id);
            return;
        }

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = input_handles
            .iter()
            .map(|input_handle| BroadcastStream::new(input_handle.receiver()))
            .collect();

        let mut combined_stream = select_all(streams);
        let context = self.context().clone();

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
                                let mut context_guard = context.write().await;
                                context_guard.handle_node_event(message).await;
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

    /// 监听策略命令
    ///
    /// 监听来自策略层的控制命令
    async fn listen_node_command(&self) {
        let (node_command_receiver, cancel_token, node_id) = self
            .with_ctx_write_async(|ctx| {
                Box::pin(async move {
                    let receiver = ctx.node_command_receiver();
                    let cancel_token = ctx.cancel_token().clone();
                    let node_id = ctx.node_id().to_string();
                    (receiver, cancel_token, node_id)
                })
            })
            .await;

        let context = self.context().clone();

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
                        let mut command_receiver_guard = node_command_receiver.lock().await;

                        if let Some(received_command) = command_receiver_guard.recv().await {
                            let mut context_guard = context.write().await;
                            context_guard.handle_node_command(received_command).await;
                        }
                    } => {}
                }
            }
        });
    }
}

// 自动为所有实现 NodeContextAccessor 且满足约束的类型实现 NodeEventListener
impl<T> NodeEventListener for T
where
    T: NodeContextAccessor,
    T::Context: super::context_trait::NodeContextExt,
{
}
