use super::super::super::strategy_event::{NodeStateLogEvent, StrategyRunningLogEvent};
use super::super::NodeEvent;
use super::BacktestNodeEvent;
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::cache::{key::KlineKey, CacheValue, KeyTrait};
use std::sync::Arc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum KlineNodeEvent {
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineUpdateEvent),

    #[strum(serialize = "state-log")]
    #[serde(rename = "state-log")]
    StateLog(NodeStateLogEvent),

    #[strum(serialize = "running-log")]
    #[serde(rename = "running-log")]
    RunningLog(StrategyRunningLogEvent),

    #[strum(serialize = "time-update")]
    #[serde(rename = "time-update")]
    TimeUpdate(TimeUpdateEvent),
}

// 类型别名
pub type KlineUpdateEvent = NodeEvent<KlineUpdatePayload>;
pub type TimeUpdateEvent = NodeEvent<TimeUpdatePayload>;

// 载荷类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineUpdatePayload {
    #[serde(rename = "configId")]
    pub config_id: i32,

    #[serde(rename = "playIndex")]
    pub play_index: i32,

    #[serde(serialize_with = "serialize_kline_cache_key")]
    #[serde(rename = "klineKey")]
    pub kline_key: KlineKey,

    #[serde(serialize_with = "serialize_kline_data")]
    #[serde(deserialize_with = "deserialize_cache_value_vec")]
    pub kline: Vec<Arc<CacheValue>>,
}

impl KlineUpdatePayload {
    pub fn new(
        config_id: i32,
        play_index: i32,
        kline_key: KlineKey,
        kline: Vec<Arc<CacheValue>>,
    ) -> Self {
        Self {
            config_id,
            play_index,
            kline_key,
            kline,
        }
    }
}

fn serialize_kline_cache_key<'de, S>(kline_key: &KlineKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let kline_key_str = kline_key.get_key_str();
    serializer.serialize_str(&kline_key_str)
}

fn serialize_kline_data<S>(
    kline_data: &Vec<Arc<CacheValue>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    let mut seq = serializer.serialize_seq(Some(kline_data.len()))?;
    kline_data
        .iter()
        .map(|cache_value| {
            let json_value = cache_value.to_json();
            seq.serialize_element(&json_value)
        })
        .collect::<Result<(), S::Error>>()?;
    seq.end()
}

// 反序列化函数
fn deserialize_cache_value_vec<'de, D>(deserializer: D) -> Result<Vec<Arc<CacheValue>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    let _: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
    Ok(Vec::new())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeUpdatePayload {
    #[serde(rename = "currentTime")]
    pub current_time: i64,
}

impl TimeUpdatePayload {
    pub fn new(current_time: i64) -> Self {
        Self { current_time }
    }
}
