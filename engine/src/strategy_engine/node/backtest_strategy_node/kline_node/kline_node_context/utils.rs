use chrono::{Datelike, Timelike, Weekday};
use star_river_core::market::KlineInterval;
use star_river_core::system::DateTimeUtc;

// 判断当前最小周期时间点是否到达更大周期的起点
// 例如：min_interval=1m, interval=1h
// 03:15:00 -> false, 03:59:00 -> false, 04:00:00 -> true
pub fn is_cross_interval(
    // _min_interval: &KlineInterval,
    interval: &KlineInterval,
    kline_datetime: &DateTimeUtc,
) -> bool {
    // 按目标周期对时间做对齐判断（以 UTC 为准）
    match interval {
        // 分钟周期：秒为0，分钟能被周期整除
        KlineInterval::Minutes1 => kline_datetime.second() == 0,
        KlineInterval::Minutes2 => kline_datetime.second() == 0 && kline_datetime.minute() % 2 == 0,
        KlineInterval::Minutes3 => kline_datetime.second() == 0 && kline_datetime.minute() % 3 == 0,
        KlineInterval::Minutes4 => kline_datetime.second() == 0 && kline_datetime.minute() % 4 == 0,
        KlineInterval::Minutes5 => kline_datetime.second() == 0 && kline_datetime.minute() % 5 == 0,
        KlineInterval::Minutes6 => kline_datetime.second() == 0 && kline_datetime.minute() % 6 == 0,
        KlineInterval::Minutes10 => kline_datetime.second() == 0 && kline_datetime.minute() % 10 == 0,
        KlineInterval::Minutes12 => kline_datetime.second() == 0 && kline_datetime.minute() % 12 == 0,
        KlineInterval::Minutes15 => kline_datetime.second() == 0 && kline_datetime.minute() % 15 == 0,
        KlineInterval::Minutes20 => kline_datetime.second() == 0 && kline_datetime.minute() % 20 == 0,
        KlineInterval::Minutes30 => kline_datetime.second() == 0 && kline_datetime.minute() % 30 == 0,

        // 小时周期：分为0、秒为0，小时能被周期整除
        KlineInterval::Hours1 => kline_datetime.second() == 0 && kline_datetime.minute() == 0,
        KlineInterval::Hours2 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 2 == 0
        }
        KlineInterval::Hours3 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 3 == 0
        }
        KlineInterval::Hours4 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 4 == 0
        }
        KlineInterval::Hours6 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 6 == 0
        }
        KlineInterval::Hours8 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 8 == 0
        }
        KlineInterval::Hours12 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 12 == 0
        }

        // 天周期：时分秒均为0
        KlineInterval::Days1 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() == 0
        }

        // 周期：周一 00:00:00（UTC）
        KlineInterval::Weeks1 => {
            kline_datetime.second() == 0
                && kline_datetime.minute() == 0
                && kline_datetime.hour() == 0
                && kline_datetime.weekday() == Weekday::Mon
        }

        // 月周期：每月1日 00:00:00（UTC）
        KlineInterval::Months1 => {
            kline_datetime.second() == 0
                && kline_datetime.minute() == 0
                && kline_datetime.hour() == 0
                && kline_datetime.day() == 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_is_cross_interval_minute_to_hour() {
        // let min_i = KlineInterval::Minutes1;
        let hour_i = KlineInterval::Hours1;

        let t1 = chrono::Utc.with_ymd_and_hms(2025, 1, 1, 3, 15, 0).unwrap();
        assert!(!is_cross_interval(&hour_i, &t1));

        let t2 = chrono::Utc.with_ymd_and_hms(2025, 1, 1, 3, 59, 0).unwrap();
        assert!(!is_cross_interval(&hour_i, &t2));

        let t3 = chrono::Utc.with_ymd_and_hms(2025, 1, 1, 4, 0, 0).unwrap();
        assert!(is_cross_interval(&hour_i, &t3));
    }
}