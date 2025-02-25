use types::indicator::SMABuffer;
use strum::Display;
use serde::{Deserialize, Serialize};
use crate::Event;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorEvent {
    #[strum(serialize = "sma-update")]
    SMAUpdate(SMABuffer),
}

impl From<IndicatorEvent> for Event {
    fn from(event: IndicatorEvent) -> Self {
        Event::Indicator(event)
    }
}
