// third-party
use chrono::{Datelike, Timelike, Weekday};

// workspace crate
use star_river_event::backtest_strategy::node_event::kline_node_event::{
    KlineNodeEvent, KlineUpdateEvent, KlineUpdatePayload,
};
use star_river_core::system::DateTimeUtc;

use star_river_core::kline::{Kline, KlineInterval};
use key::KlineKey;
use strategy_core::strategy::TimeRange;
use strategy_core::node::context_trait::NodeIdentityExt;

// current crate
use super::KlineNodeContext;

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
        KlineInterval::Hours2 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 2 == 0,
        KlineInterval::Hours3 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 3 == 0,
        KlineInterval::Hours4 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 4 == 0,
        KlineInterval::Hours6 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 6 == 0,
        KlineInterval::Hours8 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 8 == 0,
        KlineInterval::Hours12 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() % 12 == 0,

        // 天周期：时分秒均为0
        KlineInterval::Days1 => kline_datetime.second() == 0 && kline_datetime.minute() == 0 && kline_datetime.hour() == 0,

        // 周期：周一 00:00:00（UTC）
        KlineInterval::Weeks1 => {
            kline_datetime.second() == 0
                && kline_datetime.minute() == 0
                && kline_datetime.hour() == 0
                && kline_datetime.weekday() == Weekday::Mon
        }

        // 月周期：每月1日 00:00:00（UTC）
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

    total_seconds / interval_seconds
}

impl KlineNodeContext {
    pub(super) fn get_kline_update_event(
        &self,
        handle_id: String,
        config_id: i32,
        should_calculate: bool,
        kline_key: &KlineKey,
        index: i32, // 缓存索引
        kline_data: Kline,
    ) -> KlineNodeEvent {
        let payload = KlineUpdatePayload::new(config_id, index, should_calculate, kline_key.clone(), kline_data);
        KlineNodeEvent::KlineUpdate(
            KlineUpdateEvent::new(self.node_id().clone(), self.node_name().clone(), handle_id, payload).into(),
        )
    }
}
