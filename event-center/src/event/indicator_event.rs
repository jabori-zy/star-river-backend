use crate::Event;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorEvent {
    // #[strum(serialize = "sma-update")]
    // SMAUpdate(SMAIndicator),
}

impl From<IndicatorEvent> for Event {
    fn from(event: IndicatorEvent) -> Self {
        Event::Indicator(event)
    }
}
