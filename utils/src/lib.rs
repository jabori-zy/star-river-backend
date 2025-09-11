use chrono::{DateTime, FixedOffset, Utc};

// 获取utc+8的时间戳
pub fn get_utc8_timestamp_millis() -> i64 {
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&china_timezone).timestamp_millis()
}
// 获取utc+8的时间戳
pub fn get_utc8_timestamp() -> i64 {
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&china_timezone).timestamp()
}

pub fn get_utc8_datetime() -> DateTime<FixedOffset> {
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&china_timezone)
}

pub fn timestamp_to_utc8_datetime(timestamp: i64) -> DateTime<FixedOffset> {
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    DateTime::<Utc>::from_timestamp_millis(timestamp)
        .unwrap()
        .with_timezone(&china_timezone)
}

// 13位时间戳转换为utc+8的时间
pub fn timestamp_to_utc8(timestamp: i64) -> String {
    if timestamp < 1000000000000 {
        timestamp * 1000
    } else {
        timestamp
    };
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    DateTime::<Utc>::from_timestamp_millis(timestamp)
        .unwrap()
        .with_timezone(&china_timezone)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub fn seconds_to_millis(timestamp: i64) -> i64 {
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

// 生成一个唯一的batch_id
pub fn generate_batch_id() -> String {
    let timestamp = get_utc8_timestamp_millis();
    let random = rand::random::<u16>();
    format!("{}-{}", timestamp, random)
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
