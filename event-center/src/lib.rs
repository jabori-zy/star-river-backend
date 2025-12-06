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
    /// Create a new event center instance
    pub fn new() -> Self {
        Self {
            inner: EventCenterBase::new(),
        }
    }

    /// Initialize all event channels and command channels
    ///
    /// # Arguments
    /// * `event_buffer_size` - Buffer size for event channels
    /// * `command_buffer_size` - Buffer size for command channels
    pub fn init_channels(self, event_buffer_size: usize, command_buffer_size: usize) -> Self {
        Self {
            inner: self
                .inner
                .init_event_channels(event_buffer_size)
                .init_command_channels(command_buffer_size),
        }
    }

    /// Convenience method: initialize all channels with default buffer sizes
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
