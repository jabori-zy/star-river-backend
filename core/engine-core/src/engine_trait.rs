use std::{future::Future, pin::Pin, sync::Arc};

use async_trait::async_trait;
// use event_center::EventCenterSingleton;
use event_center::EventCenterSingleton;
use futures::{StreamExt, stream::select_all};
use star_river_core::error::StarRiverErrorTrait;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;

use super::EngineEventReceiver;
use crate::{
    context_trait::{EngineContextTrait, EngineEventHandler},
    state_machine::{EngineAction, EngineStateTransTrigger},
};

// 空 trait，仅用于标记
pub trait Engine: Send + Sync + 'static {}

// ============================================================================
// 引擎上下文访问器 Trait
// ============================================================================

/// 引擎上下文访问器 trait
///
/// # 关联类型
/// - `Context`: 引擎上下文类型，必须实现 `EngineContextTrait`，且其 `Action` 关联类型与 `Self::Action` 相同
/// - `Action`: 引擎动作类型，必须实现 `EngineAction`
#[async_trait]
pub trait EngineContextAccessor: Send + Sync {
    /// 引擎上下文类型
    type Context: EngineContextTrait<Action = Self::Action>;

    /// 引擎动作类型
    type Action: EngineAction;

    type Error: StarRiverErrorTrait;

    /// 获取上下文的引用
    fn context(&self) -> &Arc<RwLock<Self::Context>>;

    /// 以读锁方式访问上下文（同步闭包）
    ///
    /// # 示例
    /// ```rust,ignore
    /// let engine_name = engine.with_ctx_read(|ctx| {
    ///     ctx.engine_name().clone()
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
    /// ```rust,ignore
    /// engine.with_ctx_write(|ctx| {
    ///     // 同步操作
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
    /// ```rust,ignore
    /// engine.with_ctx_read_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.some_async_operation().await
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
    /// ```rust,ignore
    /// engine.with_ctx_write_async(|ctx| {
    ///     Box::pin(async move {
    ///         ctx.handle_command(cmd).await;
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
// 引擎生命周期 Trait
// ============================================================================

/// 引擎生命周期管理 trait
///
/// 定义引擎的生命周期相关操作（启动、停止、状态更新）
/// 依赖 `EngineContextAccessor` 来访问上下文
///
/// # 关联类型
/// - `Error`: 引擎特定的错误类型，必须实现 `StarRiverErrorTrait`
#[async_trait]
pub trait EngineLifecycle: EngineContextAccessor {
    /// 启动引擎
    ///
    /// 在引擎开始运行前调用，用于执行必要的初始化操作
    /// 具体引擎可以重写此方法来实现自定义启动逻辑
    async fn start(&self) -> Result<(), Self::Error>;

    /// 停止引擎
    ///
    /// 优雅地停止引擎，清理资源
    /// 具体引擎可以重写此方法来实现自定义清理逻辑
    async fn stop(&self) -> Result<(), Self::Error>;

    /// 更新引擎状态
    ///
    /// 处理引擎状态转换事件
    async fn update_engine_state(&self, trans_trigger: EngineStateTransTrigger) -> Result<(), Self::Error>;
}

// ============================================================================
// 引擎事件监听 Trait
// ============================================================================

/// 引擎事件监听 trait
///
/// 要求上下文类型必须同时实现 `EngineContextTrait` 和 `EngineEventHandler`
#[async_trait]
pub trait EngineEventListener: EngineContextAccessor
where
    Self::Context: EngineEventHandler,
{
    /// 监听外部事件
    async fn listen_events(&self) {
        // 使用 with_ctx_read_async 正确处理生命周期
        let (engine_name, event_receivers) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let engine_name = ctx.engine_name().clone(); // 克隆以避免生命周期问题
                    let should_receive_channels = EngineEventReceiver::get_event_receivers(&engine_name);
                    let mut event_receivers = Vec::new();
                    for channel in should_receive_channels.iter() {
                        let event_receiver = EventCenterSingleton::subscribe(channel).await.unwrap();
                        event_receivers.push(event_receiver);
                    }
                    (engine_name, event_receivers)
                })
            })
            .await;

        if event_receivers.is_empty() {
            tracing::warn!("{}: 没有事件接收器", engine_name);
            return;
        }

        let streams: Vec<_> = event_receivers.into_iter().map(|receiver| BroadcastStream::new(receiver)).collect();

        let mut combined_stream = select_all(streams);
        let context = self.context().clone();

        tracing::debug!("#[{}]: start listening events", engine_name);

        tokio::spawn(async move {
            loop {
                if let Some(received_event) = combined_stream.next().await {
                    match received_event {
                        Ok(event) => {
                            let mut context_guard = context.write().await;
                            // tracing::debug!("{}: 接收到事件: {:?}", engine_name, event);
                            context_guard.handle_event(event).await;
                        }
                        Err(e) => {
                            tracing::error!("#[{}]: receive event error: {}", engine_name, e);
                        }
                    }
                }
            }
        });
    }

    /// 监听引擎命令
    async fn listen_commands(&self) {
        let (engine_name, command_receiver) = self
            .with_ctx_read_async(|ctx| {
                Box::pin(async move {
                    let engine_name = ctx.engine_name().clone(); // 克隆以避免生命周期问题
                    let command_receiver = EventCenterSingleton::command_receiver(&engine_name.clone().into()).await.unwrap();

                    (engine_name, command_receiver)
                })
            })
            .await;
        tracing::debug!("#[{}]: start listening commands", engine_name);

        let context = self.context().clone();
        tokio::spawn(async move {
            loop {
                if let Some(received_command) = command_receiver.lock().await.recv().await {
                    let mut context_guard = context.write().await;
                    context_guard.handle_command(received_command).await;
                }
            }
        });
    }
}

impl<T> EngineEventListener for T
where
    T: EngineContextAccessor,
    T::Context: EngineEventHandler,
{
}
