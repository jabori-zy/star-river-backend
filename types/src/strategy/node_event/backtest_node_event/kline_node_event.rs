use super::super::NodeStateLogEvent;
use crate::cache::{key::KlineKey, CacheValue, KeyTrait};
use crate::strategy::node_event::BacktestNodeEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum KlineNodeEvent {
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineUpdateEvent),
    #[strum(serialize = "start-log")]
    #[serde(rename = "start-log")]
    StartLog(NodeStateLogEvent),
    #[strum(serialize = "running-log")]
    #[serde(rename = "running-log")]
    RunningLog(NodeStateLogEvent),
    #[strum(serialize = "error-log")]
    #[serde(rename = "error-log")]
    ErrorLog(NodeStateLogEvent),
}

impl From<KlineNodeEvent> for BacktestNodeEvent {
    fn from(event: KlineNodeEvent) -> Self {
        BacktestNodeEvent::KlineNode(event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineUpdateEvent {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,

    #[serde(rename = "fromNodeName")]
    pub from_node_name: String,

    #[serde(rename = "fromHandleId")]
    pub from_handle_id: String,

    #[serde(rename = "configId")]
    pub config_id: i32,

    #[serde(rename = "playIndex")]
    pub play_index: i32,

    #[serde(serialize_with = "serialize_kline_cache_key")]
    #[serde(rename = "klineKey")]
    pub kline_key: KlineKey,

    // pub kline: Vec<f64>,
    #[serde(serialize_with = "serialize_kline_data")]
    #[serde(deserialize_with = "deserialize_cache_value_vec")]
    pub kline: Vec<Arc<CacheValue>>,
    pub timestamp: i64,
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
    use serde::de::Error;
    use serde::Deserialize;

    // 这里我们简单地跳过反序列化，返回空向量
    // 在实际应用中，你可能需要根据具体需求来实现反序列化逻辑
    let _: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
    Ok(Vec::new())
}
