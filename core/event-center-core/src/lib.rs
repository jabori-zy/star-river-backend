pub mod communication;
pub mod error;
pub mod event;

use std::{collections::HashMap, sync::Arc};

pub use communication::{CommandTarget, Target};
pub use event::{Channel, Event, EventBase, EventTrait};
use tokio::sync::{Mutex, broadcast, mpsc};

/// Generic event center base structure
///
/// Type parameters:
/// - `EC`: Event Channel type (key type for event channels)
/// - `CC`: Command Target type (key type for command channels)
/// - `E`: Event type (event type, must implement EventTrait)
/// - `T`: Command type (command type, must implement CommandTarget)
pub struct EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>, // broadcast channel requires Event type to implement Clone
    T: Target<T = CC>,     // mpsc channel requires Command type to implement CommandTrait
{
    event_channels: HashMap<EC, broadcast::Sender<E>>,
    command_targets: HashMap<CC, (mpsc::Sender<T>, Arc<Mutex<mpsc::Receiver<T>>>)>,
    _black_hole: HashMap<EC, broadcast::Receiver<E>>,
}

impl<EC, CC, E, T> EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>,
    T: Target<T = CC>,
{
    pub fn new() -> Self {
        Self {
            event_channels: HashMap::new(),
            command_targets: HashMap::new(),
            _black_hole: HashMap::new(),
        }
    }

    pub fn init_event_channels_from_iter(mut self, channels: impl IntoIterator<Item = EC>, buffer_size: usize) -> Self {
        for channel in channels {
            let (tx, rx) = broadcast::channel::<E>(buffer_size);
            self.event_channels.insert(channel.clone(), tx);
            self._black_hole.insert(channel, rx);
        }
        self
    }

    /// Initialize command channels from any iterator
    ///
    /// # Arguments
    /// * `channels` - Iterator of command channels
    /// * `buffer_size` - Buffer size for each channel
    pub fn init_command_channels_from_iter(mut self, channels: impl IntoIterator<Item = CC>, buffer_size: usize) -> Self {
        for channel in channels {
            let (tx, rx) = mpsc::channel::<T>(buffer_size);
            self.command_targets.insert(channel, (tx, Arc::new(Mutex::new(rx))));
        }
        self
    }

    pub fn add_channel(mut self, channel: EC, buffer_size: usize) -> Self {
        let (tx, rx) = broadcast::channel::<E>(buffer_size);
        self.event_channels.insert(channel.clone(), tx);
        self._black_hole.insert(channel, rx);
        self
    }

    // Subscribe to events from specified channel
    pub fn subscribe(&self, channel: &EC) -> Option<broadcast::Receiver<E>> {
        self.event_channels.get(channel).map(|sender| sender.subscribe())
    }

    // Publish event to specified channel
    // Returns Result, Err contains error on send failure (number of receivers)
    pub fn publish(&self, event: E) -> Result<usize, broadcast::error::SendError<E>> {
        let channel = event.channel();
        match self.event_channels.get(channel) {
            Some(sender) => sender.send(event),
            None => Ok(0), // Channel not found, return 0 receivers
        }
    }

    pub async fn send_command(&self, command: T) -> Result<(), mpsc::error::SendError<T>> {
        let target = command.target();
        match self.command_targets.get(target) {
            Some((sender, _)) => {
                sender.send(command).await?;
                Ok(())
            }
            None => Err(mpsc::error::SendError(command)), // Channel not found, return error
        }
    }

    pub fn command_sender(&self, target: &CC) -> Option<mpsc::Sender<T>> {
        self.command_targets.get(target).map(|(sender, _)| sender.clone())
    }

    pub fn command_receiver(&self, target: &CC) -> Option<Arc<Mutex<mpsc::Receiver<T>>>> {
        self.command_targets.get(target).map(|(_, receiver)| receiver.clone())
    }
}

impl<EC, CC, E, T> EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>,
    T: Target<T = CC>,
{
    pub fn init_event_channels(self, buffer_size: usize) -> Self {
        self.init_event_channels_from_iter(EC::variants(), buffer_size)
    }
}

impl<EC, CC, E, T> EventCenterBase<EC, CC, E, T>
where
    EC: Channel,
    CC: CommandTarget,
    E: EventTrait<C = EC>,
    T: Target<T = CC>,
{
    /// Auto-initialize all command channels (suitable for enum types)
    ///
    /// Requires `CC` type to implement `EnumerableChannel` trait
    ///
    /// # Arguments
    /// * `buffer_size` - Buffer size for each channel
    pub fn init_command_channels(self, buffer_size: usize) -> Self {
        self.init_command_channels_from_iter(CC::variants(), buffer_size)
    }
}
