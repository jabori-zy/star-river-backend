// 订单事件
use crate::Event;
use serde::{Deserialize, Serialize};
use star_river_core::order::Order;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum OrderEvent {
    #[strum(serialize = "order-created")]
    #[serde(rename = "order-created")]
    OrderCreated(Order),

    #[strum(serialize = "order-updated")]
    #[serde(rename = "order-updated")]
    OrderUpdated(Order),

    #[strum(serialize = "order-canceled")]
    #[serde(rename = "order-canceled")]
    OrderCanceled(Order),

    #[strum(serialize = "order-filled")]
    #[serde(rename = "order-filled")]
    OrderFilled(Order),
}

impl From<OrderEvent> for Event {
    fn from(event: OrderEvent) -> Self {
        Event::Order(event)
    }
}
