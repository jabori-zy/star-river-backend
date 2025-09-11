use crate::Event;
use serde::{Deserialize, Serialize};
use star_river_core::position::Position;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event_name")]
pub enum PositionEvent {
    #[strum(serialize = "position-initialized")]
    #[serde(rename = "position-initialized")]
    PositionInitialized(Position), // 仓位初始化
}

impl From<PositionEvent> for Event {
    fn from(event: PositionEvent) -> Self {
        Event::Position(event)
    }
}
