use crate::define_indicator;
use chrono::{DateTime, Utc};

// AD                   Chaikin A/D Line #钱德动量线
define_indicator!(AD,
    params => [],
    output => [(datetime: DateTime<Utc>), (ad: f64)],
);

// ADOSC                Chaikin A/D Oscillator #钱德动量振荡器
define_indicator!(ADOSC,
    params => [(fast_period: i32), (slow_period: i32)],
    output => [(datetime: DateTime<Utc>), (adosc: f64)],
);

// OBV                  On Balance Volume #能量潮
define_indicator!(OBV,
    params => [],
    output => [(datetime: DateTime<Utc>), (obv: f64)],
);
