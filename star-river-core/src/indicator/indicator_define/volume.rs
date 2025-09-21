use crate::define_indicator;
use chrono::{DateTime, Utc};

// AD                   Chaikin A/D Line #钱德动量线
define_indicator!(AD,
    params => [],
    output => [(datetime: DateTime<Utc>), (ad: Option<f64>)],
);

// ADOSC                Chaikin A/D Oscillator #钱德动量振荡器
define_indicator!(ADOSC,
    params => [(fast_period: i32), (slow_period: i32)],
    output => [(datetime: DateTime<Utc>), (adosc: Option<f64>)],
);

// OBV                  On Balance Volume #能量潮
define_indicator!(OBV,
    params => [],
    output => [(datetime: DateTime<Utc>), (obv: Option<f64>)],
);
