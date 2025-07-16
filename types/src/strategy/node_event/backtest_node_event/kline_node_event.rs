use serde::{Deserialize, Serialize};
use strum::Display;
use crate::cache::{key::BacktestKlineKey, KeyTrait, CacheValue};
use std::sync::Arc;
use crate::strategy::node_event::BacktestNodeEvent;




#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum KlineNodeEvent {
    #[strum(serialize = "kline-update")]
    #[serde(rename = "kline-update")]
    KlineUpdate(KlineUpdateEvent),
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

    #[serde(rename = "playIndex")]
    pub play_index: u32,

    #[serde(serialize_with = "serialize_kline_cache_key")]
    #[serde(rename = "klineKey")]
    pub kline_key: BacktestKlineKey,

    // pub kline: Vec<f64>,
    #[serde(serialize_with = "serialize_kline_data")]
    pub kline: Vec<Arc<CacheValue>>,
    pub timestamp: i64,
}


fn serialize_kline_cache_key<'de, S>(kline_key: &BacktestKlineKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let kline_key_str = kline_key.get_key_str();
    serializer.serialize_str(&kline_key_str)
}

fn serialize_kline_data<S>(kline_data: &Vec<Arc<CacheValue>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    
    let mut seq = serializer.serialize_seq(Some(kline_data.len()))?;
    kline_data.iter().map(|cache_value| {
        let json_value = cache_value.to_json();
        seq.serialize_element(&json_value)
    }).collect::<Result<(), S::Error>>()?;
    seq.end()
}

    
    
