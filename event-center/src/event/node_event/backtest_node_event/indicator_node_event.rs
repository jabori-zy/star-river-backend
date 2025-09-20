use super::super::NodeEvent;
#[allow(unused_imports)]
use super::{deserialize_cache_value_vec, serialize_cache_value_vec};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::cache::key::IndicatorKey;
use star_river_core::cache::{CacheValue, CacheItem};
use star_river_core::cache::KeyTrait;
use star_river_core::indicator::IndicatorConfig;
use star_river_core::market::{Exchange, KlineInterval};
use std::sync::Arc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
pub enum IndicatorNodeEvent {
    #[strum(serialize = "indicator-update-event")]
    #[serde(rename = "indicator-update-event")]
    IndicatorUpdate(IndicatorUpdateEvent),
}

// 类型别名
pub type IndicatorUpdateEvent = NodeEvent<IndicatorUpdatePayload>;

// 载荷类型定义
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

    #[serde(rename = "indicatorSeries")]
    #[serde(serialize_with = "serialize_indicator_data")]
    pub indicator_series: Vec<Arc<CacheValue>>,

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
        indicator_series: Vec<Arc<CacheValue>>,
        play_index: i32,
    ) -> Self {
        Self {
            exchange,
            symbol,
            interval,
            config_id,
            indicator_config,
            indicator_key,
            indicator_series,
            play_index,
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
