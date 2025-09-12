use crate::define_indicator;
use chrono::{DateTime, FixedOffset};

// ATR                  Average True Range #平均真实波幅 #平均真实波幅
define_indicator!(ATR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<FixedOffset>), (atr: f64)],
);

// NATR                 Normalized Average True Range #归一化平均真实波幅
define_indicator!(NATR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<FixedOffset>), (natr: f64)],
);

// TRANGE               True Range #真实波幅
define_indicator!(TRANGE,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<FixedOffset>), (trange: f64)],
);
