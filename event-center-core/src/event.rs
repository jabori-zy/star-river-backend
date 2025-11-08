use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::fmt::Debug;
use std::hash::Hash;
use strum::IntoEnumIterator;
use std::fmt::Display;



/// Trait for channel types that can enumerate all their variants
/// 用户自定义的通道类型需要实现此 trait
pub trait Channel: Eq + Hash + Clone + IntoEnumIterator + Display {
    fn variants() -> Vec<Self>;
}



pub trait EventTrait: Clone + Debug + Send + Sync + 'static {
    type C: Channel;
    // 获取事件所属的通道
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


// 泛型事件结构
// 类似 NodeEvent<T>，包含 EventBase 和 payload
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

// 使用 Deref 允许直接访问 payload 字段
impl<T> Deref for Event<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.payload
    }
}



