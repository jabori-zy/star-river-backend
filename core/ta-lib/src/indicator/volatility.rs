use chrono::{DateTime, Utc};

use crate::define_indicator;

// ATR                  Average True Range
define_indicator!(ATR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (atr: Option<f64>)],
);

// NATR                 Normalized Average True Range
define_indicator!(NATR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (natr: Option<f64>)],
);

// TRANGE               True Range
define_indicator!(TRANGE,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (trange: Option<f64>)],
);
