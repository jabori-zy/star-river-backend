use crate::system::DateTimeUtc;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use std::sync::LazyLock;

// UTC+8 时区常量，使用 LazyLock 延迟初始化
static UTC8_OFFSET: LazyLock<FixedOffset> = LazyLock::new(|| FixedOffset::east_opt(8 * 3600).expect("Invalid UTC+8 offset"));

// 获取utc+8的时间戳
pub fn get_utc8_timestamp_millis() -> i64 {
    Utc::now().with_timezone(&*UTC8_OFFSET).timestamp_millis()
}

// 获取utc+8的时间戳
pub fn get_utc8_timestamp() -> i64 {
    Utc::now().with_timezone(&*UTC8_OFFSET).timestamp()
}

// 使用项目统一的错误类型
use crate::error::datetime_error::*;

/// 时间戳转换为 UTC+8 时间（返回 Result）
/// 支持秒级和毫秒级时间戳的自动识别
// pub fn timestamp_to_utc8_datetime(timestamp: i64) -> Result<DateTimeUtc, DateTimeError> {

//     // 验证时间戳有效性
//     if timestamp <= 0 {
//         return Err(InvalidTimestampSnafu {
//             timestamp
//         }.build());
//     }

//     // 更好的时间戳识别逻辑
//     let timestamp_millis = seconds_to_millis(timestamp);

//     // 使用 Result 处理错误
//     Utc.timestamp_millis_opt(timestamp_millis)
//         .single()
//         .map(|utc_time| utc_time.with_timezone(&*UTC8_OFFSET))
//         .ok_or_else(|| TransformTimestampFailedSnafu {
//             timestamp
//         }.build())
// }

/// 时间戳转换为 UTC+8 时间（旧版本兼容，会 panic）
/// 不推荐使用，建议迁移到 timestamp_to_utc8_datetime
// pub fn timestamp_to_utc8_datetime_legacy(timestamp: i64) -> DateTimeUtc {
//     timestamp_to_utc8_datetime(timestamp)
//         .unwrap_or_else(|e| panic!("Failed to convert timestamp: {}", e))
// }

// 13位时间戳转换为utc+8的时间
pub fn timestamp_to_utc8(timestamp: i64) -> String {
    // 修复：实际使用转换后的时间戳
    let timestamp_millis = if timestamp < 1000000000000 {
        timestamp * 1000 // 秒转毫秒
    } else {
        timestamp
    };

    DateTime::<Utc>::from_timestamp_millis(timestamp_millis)
        .unwrap()
        .with_timezone(&*UTC8_OFFSET)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn seconds_to_millis(timestamp: i64) -> i64 {
    // 检查时间戳的位数来判断是否为秒级
    // 13位左右是毫秒级时间戳（2020年代的毫秒时间戳约为13位）
    // 10位左右是秒级时间戳（2020年代的秒时间戳约为10位）
    if timestamp > 0 && timestamp < 10_000_000_000 {
        // 小于10位数的时间戳（直到2286年）视为秒级，转为毫秒级
        timestamp * 1000
    } else {
        timestamp
    }
}

// 驼峰命名转换为下划线命名
pub fn camel_to_snake(name: &str) -> String {
    name.chars()
        .enumerate()
        .map(|(i, c)| {
            if i > 0 && c.is_uppercase() {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("")
}
