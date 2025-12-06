use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

/// Trait for channel types that can enumerate all their variants
/// User-defined channel types need to implement this trait
pub trait Channel: Eq + Hash + Clone + IntoEnumIterator + Display {
    fn variants() -> Vec<Self>;
}

pub trait EventTrait: Clone + Debug + Send + Sync + 'static {
    type C: Channel;
    // Get the channel this event belongs to
    fn channel(&self) -> &Self::C;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct EventBase {
    pub datetime: chrono::DateTime<Utc>,
}

impl EventBase {
    pub fn new() -> Self {
        Self { datetime: Utc::now() }
    }

    pub fn datetime(&self) -> chrono::DateTime<Utc> {
        self.datetime
    }
}

// Generic event structure
// Similar to NodeEvent<T>, contains EventBase and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T> {
    #[serde(flatten)]
    pub event_base: EventBase,
    #[serde(flatten)]
    pub payload: T,
}

impl<T> Event<T> {
    pub fn new(payload: T) -> Self {
        let event_base = EventBase::new();
        Self { event_base, payload }
    }

    pub fn datetime(&self) -> chrono::DateTime<Utc> {
        self.event_base.datetime()
    }
}

// Use Deref to allow direct access to payload field
impl<T> Deref for Event<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}
