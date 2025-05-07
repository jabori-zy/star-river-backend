use serde::{Serialize, Deserialize};
use strum::Display;
use crate::market_event::KlineSeriesInfo;
use crate::market_event::KlineInfo;


#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "message_type")]
pub enum ChartMessage {
    KlineSeriesUpdate(KlineSeriesInfo),
    KlineUpdate(KlineInfo),
}
