pub mod communication;
// pub mod error;
pub mod event;
pub mod singleton;

pub use communication::{CommandTargetEngine, EngineCommand};
pub use event::Event;
use event_center_core::EventCenterBase;
pub use event_center_core::error::*;
pub use singleton::EventCenterSingleton;

use crate::event::Channel;

pub struct EventCenter {
    inner: EventCenterBase<Channel, CommandTargetEngine, Event, EngineCommand>,
}

impl EventCenter {
    /// 创建一个新的事件中心实例
    pub fn new() -> Self {
        Self {
            inner: EventCenterBase::new(),
        }
    }

    /// 初始化所有事件通道和命令通道
    ///
    /// # Arguments
    /// * `event_buffer_size` - 事件通道的缓冲区大小
    /// * `command_buffer_size` - 命令通道的缓冲区大小
    pub fn init_channels(self, event_buffer_size: usize, command_buffer_size: usize) -> Self {
        Self {
            inner: self
                .inner
                .init_event_channels(event_buffer_size)
                .init_command_channels(command_buffer_size),
        }
    }

    /// 便捷方法：使用默认缓冲区大小初始化所有通道
    pub fn init_with_default(self) -> Self {
        self.init_channels(100, 100)
    }
}

impl Default for EventCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for EventCenter {
    type Target = EventCenterBase<Channel, CommandTargetEngine, Event, EngineCommand>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for EventCenter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
