use chrono::{DateTime, Utc};
use derive_more::From;
use key::{KeyTrait, KlineKey};
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::{CycleId, HandleId, NodeId, NodeName},
    kline::Kline,
};
use strategy_core::event::node::NodeEvent;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum KlineNodeEvent {
    #[strum(serialize = "kline-update-event")]
    #[serde(rename = "kline-update-event")]
    KlineUpdate(KlineUpdateEvent),
}

impl KlineNodeEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            KlineNodeEvent::KlineUpdate(event) => event.cycle_id(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            KlineNodeEvent::KlineUpdate(event) => event.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            KlineNodeEvent::KlineUpdate(event) => event.node_id(),
        }
    }
    pub fn node_name(&self) -> &NodeName {
        match self {
            KlineNodeEvent::KlineUpdate(event) => event.node_name(),
        }
    }
    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            KlineNodeEvent::KlineUpdate(event) => event.output_handle_id(),
        }
    }
}

// Type aliases
pub type KlineUpdateEvent = NodeEvent<KlineUpdatePayload>;

// Payload type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KlineUpdatePayload {
    pub config_id: i32,
    pub should_calculate: bool, // Whether calculation is needed
    #[serde(serialize_with = "serialize_kline_key")]
    pub kline_key: KlineKey,
    pub kline: Kline,
}

impl KlineUpdatePayload {
    pub fn new(config_id: i32, should_calculate: bool, kline_key: KlineKey, kline: Kline) -> Self {
        Self {
            config_id,
            should_calculate,
            kline_key,
            kline,
        }
    }
}

fn serialize_kline_key<'de, S>(kline_key: &KlineKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let kline_key_str = kline_key.key_str();
    serializer.serialize_str(&kline_key_str)
}
