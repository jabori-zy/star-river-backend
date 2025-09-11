use crate::define_indicator;

// AD                   Chaikin A/D Line #钱德动量线
define_indicator!(AD,
    params => [],
    output => [(timestamp: i64), (ad: f64)],
);

// ADOSC                Chaikin A/D Oscillator #钱德动量振荡器
define_indicator!(ADOSC,
    params => [(fast_period: i32), (slow_period: i32)],
    output => [(timestamp: i64), (adosc: f64)],
);

// OBV                  On Balance Volume #能量潮
define_indicator!(OBV,
    params => [],
    output => [(timestamp: i64), (obv: f64)],
);
