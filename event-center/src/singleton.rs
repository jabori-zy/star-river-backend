use crate::communication::engine::{EngineCommand, EngineCommandReceiver, EngineCommandSender};
use crate::event_center_error::*;
use crate::{Channel, CommandPublisher, Event, EventCenter, EventCenterError, EventPublisher, EventReceiver};
use star_river_core::engine::EngineName;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::{Mutex, RwLock};

/// 全局事件中心单例
static EVENT_CENTER_INSTANCE: OnceLock<Arc<RwLock<EventCenter>>> = OnceLock::new();

/// 单例事件中心管理器
pub struct EventCenterSingleton;

impl EventCenterSingleton {
    /// 初始化事件中心
    /// 必须在使用其他方法之前调用
    pub async fn init() -> Result<(), EventCenterError> {
        let event_center = Arc::new(RwLock::new(EventCenter::new().init_channel().await));

        EVENT_CENTER_INSTANCE.set(event_center).map_err(|_| {
            EventCenterError::EventCenterInstanceAlreadyInitialized {
                backtrace: snafu::Backtrace::capture(),
            }
        })?;

        tracing::info!("EventCenter singleton initialized successfully");
        Ok(())
    }

    /// 检查是否已初始化
    pub fn is_initialized() -> bool {
        EVENT_CENTER_INSTANCE.get().is_some()
    }

    /// 获取事件发布器
    pub async fn publisher() -> Result<EventPublisher, EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        Ok(center.get_event_publisher())
    }

    /// 获取命令发布器
    pub async fn command_publisher() -> Result<CommandPublisher, EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        Ok(center.get_command_publisher())
    }

    /// 订阅事件通道
    pub async fn subscribe(channel: &Channel) -> Result<EventReceiver, EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        center.subscribe(channel).await
    }

    /// 直接发布事件（优化版本 - 去除 HashMap 锁）
    pub async fn publish(event: Event) -> Result<(), EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        let channel = event.get_channel();

        // 直接访问 HashMap，无需额外锁
        let sender = center.broadcast_channels.get(&channel).ok_or_else(|| {
            ChannelNotFoundSnafu {
                channel: channel.to_string(),
            }
            .build()
        })?;

        match event.clone() {
            Event::Strategy(_strategy_event) => {
                // tracing::debug!("发布事件: 事件通道: {:?}", channel);
            }
            _ => {
                // tracing::debug!("发布事件: 事件通道: {:?}", channel);
            }
        }

        sender.send(event)?;

        Ok(())
    }

    /// 直接发送命令（便捷方法）
    pub async fn send_command(command: EngineCommand) -> Result<(), EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        let engine_name = command.get_engine_name();
        let (sender, _receiver) = center.command_channels.get(&engine_name).ok_or_else(|| {
            EngineCommandSenderNotFoundSnafu {
                engine_name: engine_name.to_string(),
            }
            .build()
        })?;

        sender.send(command).await?;
        Ok(())
    }

    /// 获取指定引擎的命令接收器
    pub async fn get_command_receiver(
        engine_name: &EngineName,
    ) -> Result<Arc<Mutex<EngineCommandReceiver>>, EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        center.get_command_receiver(engine_name).await
    }

    /// 获取指定引擎的命令发送器
    pub async fn get_command_sender(engine_name: EngineName) -> Result<EngineCommandSender, EventCenterError> {
        let instance = EVENT_CENTER_INSTANCE
            .get()
            .ok_or_else(|| EventCenterInstanceNotInitializedSnafu {}.build())?;

        let center = instance.read().await;
        center.get_command_sender(engine_name)
    }

    /// 获取所有可用通道列表
    pub fn get_all_channels() -> Vec<Channel> {
        Channel::get_all_channels()
    }
}
