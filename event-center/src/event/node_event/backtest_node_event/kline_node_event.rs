use super::super::super::strategy_event::{NodeStateLogEvent, StrategyRunningLogEvent};
use super::super::NodeEvent;
use chrono::{DateTime, Utc};
use derive_more::From;
use serde::{Deserialize, Serialize};
use star_river_core::custom_type::PlayIndex;
use star_river_core::key::KeyTrait;
use star_river_core::key::key::KlineKey;
use star_river_core::market::Kline;
use star_river_core::market::QuantData;
use std::sync::Arc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, From)]
#[serde(tag = "event")]
pub enum KlineNodeEvent {
    #[strum(serialize = "kline-update-event")]
    #[serde(rename = "kline-update-event")]
    KlineUpdate(KlineUpdateEvent),

    #[strum(serialize = "state-log-event")]
    #[serde(rename = "state-log-event")]
    StateLog(NodeStateLogEvent),

    #[strum(serialize = "running-log-event")]
    #[serde(rename = "running-log-event")]
    RunningLog(StrategyRunningLogEvent),

    #[strum(serialize = "time-update-event")]
    #[serde(rename = "time-update-event")]
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
    pub play_index: PlayIndex,

    #[serde(rename = "shouldCalculate")]
    pub should_calculate: bool, // 是否需要计算

    #[serde(serialize_with = "serialize_kline_key")]
    #[serde(rename = "klineKey")]
    pub kline_key: KlineKey,

    // #[serde(serialize_with = "serialize_kline_data")]
    // #[serde(deserialize_with = "deserialize_cache_value_vec")]
    #[serde(rename = "kline")]
    pub kline: Kline,
}

impl KlineUpdatePayload {
    pub fn new(config_id: i32, play_index: PlayIndex, should_calculate: bool, kline_key: KlineKey, kline: Kline) -> Self {
        Self {
            config_id,
            play_index,
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
    let kline_key_str = kline_key.get_key_str();
    serializer.serialize_str(&kline_key_str)
}

fn serialize_kline_data<S>(kline_data: &Vec<Kline>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    let mut seq = serializer.serialize_seq(Some(kline_data.len()))?;
    kline_data
        .iter()
        .map(|v| {
            let json_value = v.to_json();
            seq.serialize_element(&json_value)
        })
        .collect::<Result<(), S::Error>>()?;
    seq.end()
}

// 反序列化函数
fn deserialize_cache_value_vec<'de, D>(deserializer: D) -> Result<Vec<Kline>, D::Error>
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
    pub current_time: DateTime<Utc>,
}

impl TimeUpdatePayload {
    pub fn new(current_time: DateTime<Utc>) -> Self {
        Self { current_time }
    }
}
