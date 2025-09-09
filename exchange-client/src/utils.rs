use chrono::{FixedOffset, Utc};
use serde::Deserialize;

// 获取utc+8的时间戳
pub fn get_utc8_timestamp() -> i64 {
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&china_timezone).timestamp_millis()
}

// 将字符串转换为f64
pub fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}
