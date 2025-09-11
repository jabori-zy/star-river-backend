use chrono::{DateTime, FixedOffset};

/// UTC+8 时区常量 (东八区)
pub const UTC8_OFFSET: FixedOffset = match FixedOffset::east_opt(8 * 3600) {
    Some(offset) => offset,
    None => panic!("Invalid UTC+8 offset"),
};

/// UTC+8 时间类型别名
pub type Utc8DateTime = DateTime<FixedOffset>;
