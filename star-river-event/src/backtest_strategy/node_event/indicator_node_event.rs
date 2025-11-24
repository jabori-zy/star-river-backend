// #[allow(unused_imports)]
// use super::super::super::deserialize_cache_value_vec;
// #[allow(unused_imports)]
// use super::super::super::serialize_cache_value_vec;
use chrono::{DateTime, Utc};
use derive_more::From;
use key::{IndicatorKey, KeyTrait};
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::{CycleId, HandleId, NodeId, NodeName},
    exchange::Exchange,
    kline::KlineInterval,
};
use strategy_core::event::node::NodeEvent;
use strum::Display;
use ta_lib::{Indicator, IndicatorConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum IndicatorNodeEvent {
    #[strum(serialize = "indicator-update-event")]
    #[serde(rename = "indicator-update-event")]
    IndicatorUpdate(IndicatorUpdateEvent),
}

impl IndicatorNodeEvent {
    pub fn cycle_id(&self) -> CycleId {
        match self {
            IndicatorNodeEvent::IndicatorUpdate(event) => event.cycle_id(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        match self {
            IndicatorNodeEvent::IndicatorUpdate(event) => event.datetime(),
        }
    }

    pub fn node_id(&self) -> &NodeId {
        match self {
            IndicatorNodeEvent::IndicatorUpdate(event) => event.node_id(),
        }
    }

    pub fn node_name(&self) -> &NodeName {
        match self {
            IndicatorNodeEvent::IndicatorUpdate(event) => event.node_name(),
        }
    }

    pub fn output_handle_id(&self) -> &HandleId {
        match self {
            IndicatorNodeEvent::IndicatorUpdate(event) => event.output_handle_id(),
        }
    }
}

// Type alias
pub type IndicatorUpdateEvent = NodeEvent<IndicatorUpdatePayload>;

// Payload type definition
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorUpdatePayload {
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub config_id: i32,
    pub indicator_config: IndicatorConfig,

    #[serde(serialize_with = "serialize_indicator_key")]
    pub indicator_key: IndicatorKey,
    #[serde(serialize_with = "serialize_indicator_data")]
    pub indicator_value: Indicator,
}

impl IndicatorUpdatePayload {
    pub fn new(
        exchange: Exchange,
        symbol: String,
        interval: KlineInterval,
        config_id: i32,
        indicator_config: IndicatorConfig,
        indicator_key: IndicatorKey,
        indicator_value: Indicator,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            config_id,
            indicator_config,
            indicator_key,
            indicator_value,
        }
    }
}

#[allow(dead_code)]
fn serialize_indicator_key<'de, S>(indicator_key: &IndicatorKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let indicator_key_str = indicator_key.key_str();
    serializer.serialize_str(&indicator_key_str)
}

fn serialize_indicator_data<S>(indicator_data: &Indicator, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let json_value = indicator_data.to_json();
    json_value.serialize(serializer)
}
