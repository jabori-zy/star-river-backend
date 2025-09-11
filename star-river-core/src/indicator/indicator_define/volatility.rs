use crate::define_indicator;

// ATR                  Average True Range #平均真实波幅 #平均真实波幅
define_indicator!(ATR,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (atr: f64)],
);

// NATR                 Normalized Average True Range #归一化平均真实波幅
define_indicator!(NATR,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (natr: f64)],
);

// TRANGE               True Range #真实波幅
define_indicator!(TRANGE,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (trange: f64)],
);
