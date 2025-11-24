// #[allow(unused_imports)]
// use super::super::super::deserialize_cache_value_vec;
// #[allow(unused_imports)]
// use super::super::super::serialize_cache_value_vec;
use derive_more::From;
use key::{IndicatorKey, KeyTrait};
use serde::{Deserialize, Serialize};
use star_river_core::{
    custom_type::{HandleId, NodeId, NodeName},
    exchange::Exchange,
    kline::KlineInterval,
};
use strategy_core::event::node::NodeEvent;
use strum::Display;
use ta_lib::{Indicator, IndicatorConfig};

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
pub enum IndicatorNodeEvent {
    #[strum(serialize = "indicator-update-event")]
    #[serde(rename = "indicator-update-event")]
    IndicatorUpdate(IndicatorUpdateEvent),
}

impl IndicatorNodeEvent {
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
pub struct IndicatorUpdatePayload {
    #[serde(rename = "exchange")]
    pub exchange: Exchange,

    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "interval")]
    pub interval: KlineInterval,

    #[serde(rename = "configId")]
    pub config_id: i32,

    #[serde(rename = "indicatorConfig")]
    pub indicator_config: IndicatorConfig,

    #[serde(rename = "indicatorKey")]
    #[serde(serialize_with = "serialize_indicator_cache_key")]
    pub indicator_key: IndicatorKey,

    #[serde(rename = "indicatorValue")]
    #[serde(serialize_with = "serialize_indicator_data")]
    pub indicator_value: Indicator,

    #[serde(rename = "playIndex")]
    pub play_index: i32,
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
        play_index: i32,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            config_id,
            indicator_config,
            indicator_key,
            indicator_value,
            play_index,
        }
    }
}

#[allow(dead_code)]
fn serialize_indicator_cache_key<'de, S>(indicator_cache_key: &IndicatorKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let indicator_cache_key_str = indicator_cache_key.key_str();
    serializer.serialize_str(&indicator_cache_key_str)
}

// #[allow(dead_code)]
// fn serialize_indicator_data<S>(
//     indicator_data: &Arc<CacheValue>,
//     serializer: S,
// ) -> Result<S::Ok, S::Error>
// where
//     S: serde::Serializer,
// {
//     use serde::ser::SerializeSeq;

//     let mut seq = serializer.serialize_seq(Some(indicator_data.len()))?;
//     indicator_data
//         .iter()
//         .map(|indicator_value| {
//             let json_value = indicator_value.to_json();
//             seq.serialize_element(&json_value)
//         })
//         .collect::<Result<(), S::Error>>()?;
//     seq.end()
// }

fn serialize_indicator_data<S>(indicator_data: &Indicator, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let json_value = indicator_data.to_json();
    json_value.serialize(serializer)
}
