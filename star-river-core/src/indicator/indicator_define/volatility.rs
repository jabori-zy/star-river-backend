use crate::define_indicator;
use crate::error::star_river_error::*;
use chrono::{DateTime, Utc};

// ATR                  Average True Range #平均真实波幅 #平均真实波幅
define_indicator!(ATR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (atr: Option<f64>)],
);

// NATR                 Normalized Average True Range #归一化平均真实波幅
define_indicator!(NATR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (natr: Option<f64>)],
);

// TRANGE               True Range #真实波幅
define_indicator!(TRANGE,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (trange: Option<f64>)],
);
