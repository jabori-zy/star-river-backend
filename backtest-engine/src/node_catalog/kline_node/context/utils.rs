// third-party
use chrono::{Datelike, Timelike, Weekday};
// workspace crate
use star_river_core::system::DateTimeUtc;
use star_river_core::{kline::KlineInterval, system::TimeRange};

// Determine if current minimum interval time point has reached the start of a larger period
// Example: min_interval=1m, interval=1h
// 03:15:00 -> false, 03:59:00 -> false, 04:00:00 -> true
pub fn is_cross_interval(
    // _min_interval: &KlineInterval,
    interval: &KlineInterval,
    kline_datetime: &DateTimeUtc,
) -> bool {
    // Align time according to target period (based on UTC)
    match interval {
        // Minute periods: second is 0, minute divisible by period
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

        // Hour periods: minute is 0, second is 0, hour divisible by period
        KlineInterval::Hours1 => kline_datetime.second() == 0 && kline_datetime.minute() == 0,
        KlineInterval::Hours2 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 2 == 0,
        KlineInterval::Hours3 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 3 == 0,
        KlineInterval::Hours4 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 4 == 0,
        KlineInterval::Hours6 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 6 == 0,
        KlineInterval::Hours8 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 8 == 0,
        KlineInterval::Hours12 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 12 == 0,

        // Day period: hour, minute, second all 0
        KlineInterval::Days1 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() == 0,

        // Week period: Monday 00:00:00 (UTC)
        KlineInterval::Weeks1 => {
            kline_datetime.second() == 0
                && kline_datetime.minute() == 0
                && kline_datetime.hour() == 0
                && kline_datetime.weekday() == Weekday::Mon
        }

        // Month period: 1st of each month 00:00:00 (UTC)
        KlineInterval::Months1 => {
            kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() == 0 && kline_datetime.day() == 1
        }
    }
}

pub fn bar_number(time_range: &TimeRange, interval: &KlineInterval) -> i64 {
    let total_seconds = time_range.duration().num_seconds();
    let interval_seconds = interval.to_seconds();

    if interval_seconds <= 0 {
        return 0;
    }

    total_seconds / interval_seconds as i64
}
