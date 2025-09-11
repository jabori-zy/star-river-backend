use super::{deserialize_cache_value_vec, serialize_cache_value_vec};
use serde::{Deserialize, Serialize};
use star_river_core::cache::key::IndicatorKey;
use star_river_core::cache::CacheValue;
use star_river_core::cache::KeyTrait;
use star_river_core::indicator::IndicatorConfig;
use star_river_core::market::{Exchange, KlineInterval};
use std::sync::Arc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum IndicatorNodeEvent {
    #[strum(serialize = "indicator_update")]
    #[serde(rename = "indicator_update")]
    LiveIndicatorUpdate(LiveIndicatorUpdateEvent), // 实盘指标更新
    IndicatorUpdate(IndicatorUpdateEvent), // 回测指标更新
}

// 指标消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveIndicatorUpdateEvent {
    pub from_node_id: String,
    pub from_node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator_config: IndicatorConfig,
    #[serde(serialize_with = "serialize_cache_value_vec")]
    #[serde(deserialize_with = "deserialize_cache_value_vec")]
    pub indicator_series: Vec<Arc<CacheValue>>,
    pub message_timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndicatorUpdateEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromNodeHandleId")]
    pub from_handle_id: String,

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

    #[serde(rename = "indicatorSeries")]
    #[serde(serialize_with = "serialize_indicator_data")]
    pub indicator_series: Vec<Arc<CacheValue>>,

    #[serde(rename = "playIndex")]
    pub play_index: i32,

    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

fn serialize_indicator_cache_key<'de, S>(
    indicator_cache_key: &IndicatorKey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let indicator_cache_key_str = indicator_cache_key.get_key_str();
    serializer.serialize_str(&indicator_cache_key_str)
}

fn serialize_indicator_data<S>(
    indicator_data: &Vec<Arc<CacheValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    let mut seq = serializer.serialize_seq(Some(indicator_data.len()))?;
    indicator_data
        .iter()
        .map(|indicator_value| {
            let json_value = indicator_value.to_json();
            seq.serialize_element(&json_value)
        })
        .collect::<Result<(), S::Error>>()?;
    seq.end()
}
