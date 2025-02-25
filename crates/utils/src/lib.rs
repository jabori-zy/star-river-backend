use chrono::{Utc, FixedOffset, DateTime};


// 获取utc+8的时间戳
pub fn get_utc8_timestamp() -> i64 {
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&china_timezone).timestamp_millis()

}


// 13位时间戳转换为utc+8的时间
pub fn timestamp_to_utc8(timestamp: i64) -> String {
    if timestamp < 1000000000000 {
        timestamp * 1000
    } else {

        timestamp
    };
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    DateTime::<Utc>::from_timestamp_millis(timestamp).unwrap().with_timezone(&china_timezone).format("%Y-%m-%d %H:%M:%S").to_string()
}
