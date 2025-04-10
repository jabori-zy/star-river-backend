// 订单事件
use serde::{Deserialize, Serialize};
use types::order::Order;
use strum::Display;
use crate::Event;

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
}


impl From<OrderEvent> for Event {
    fn from(event: OrderEvent) -> Self {
        Event::Order(event)
    }
}