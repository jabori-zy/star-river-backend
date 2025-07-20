use crate::define_indicator;
use crate::indicator::{PriceSource, MAType};




define_indicator!(MACD,
    params => [(fast_period: i32), (slow_period: i32), (signal_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);

define_indicator!(MA,
    params => [(time_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (ma: f64)],
);

define_indicator!(BBands,
    params => [(time_period: i32), (dev_up: f64), (dev_down: f64), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (upper: f64), (middle: f64), (lower: f64)],
);

define_indicator!(RSI,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rsi: f64)],
);
