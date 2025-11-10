pub mod system_config;

pub use system_config::*;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub type DateTimeUtc = DateTime<Utc>;

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(rename = "startDate")]
    pub start_date: DateTime<Utc>, // 开始日期
    #[serde(rename = "endDate")]
    pub end_date: DateTime<Utc>, // 结束日期
}

impl TimeRange {
    pub fn new(start_date_str: String, end_date_str: String) -> Self {
        use chrono::NaiveDateTime;

        // 尝试解析RFC 3339格式（如：1971-01-01T00:00:00Z）
        let start_date = match DateTimeUtc::from_str(&start_date_str) {
            Ok(dt) => dt,
            Err(_) => {
                // 如果RFC 3339格式失败，尝试解析"YYYY-MM-DD HH:MM:SS"格式
                match NaiveDateTime::parse_from_str(&start_date_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_dt) => naive_dt.and_utc(),
                    Err(e) => panic!("Failed to parse start_date '{}': {}", start_date_str, e),
                }
            }
        };

        let end_date = match DateTimeUtc::from_str(&end_date_str) {
            Ok(dt) => dt,
            Err(_) => {
                // 如果RFC 3339格式失败，尝试解析"YYYY-MM-DD HH:MM:SS"格式
                match NaiveDateTime::parse_from_str(&end_date_str, "%Y-%m-%d %H:%M:%S") {
                    Ok(naive_dt) => naive_dt.and_utc(),
                    Err(e) => panic!("Failed to parse end_date '{}': {}", end_date_str, e),
                }
            }
        };

        Self { start_date, end_date }
    }

    pub fn duration(&self) -> Duration {
        self.end_date.signed_duration_since(self.start_date)
    }
}

impl fmt::Display for TimeRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ~ {}", self.start_date, self.end_date)
    }
}

pub fn deserialize_time_range<'de, D>(deserializer: D) -> Result<TimeRange, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let time_range_value = serde_json::Value::deserialize(deserializer)?;

    if let serde_json::Value::Object(map) = time_range_value {
        let start_date_str = map.get("startDate").and_then(|v| v.as_str());
        let end_date_str = map.get("endDate").and_then(|v| v.as_str());

        if let (Some(start), Some(end)) = (start_date_str, end_date_str) {
            match (
                //前端返回的2025-09-13 00:00:00 +08:00格式 自带时区，解析为DateTime<Utc>
                DateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S %z"),
                DateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S %z"),
            ) {
                (Ok(start_with_tz), Ok(end_with_tz)) => {
                    // 转换为UTC时区
                    let start_date = start_with_tz.with_timezone(&Utc);
                    let end_date = end_with_tz.with_timezone(&Utc);
                    return Ok(TimeRange { start_date, end_date });
                }
                _ => {
                    return Err(serde::de::Error::custom(
                        "can't parse date format, expected format: YYYY-MM-DD HH:MM:SS +TZ:TZ",
                    ));
                }
            }
        }
    }

    Err(serde::de::Error::custom("date format is incorrect"))
}
