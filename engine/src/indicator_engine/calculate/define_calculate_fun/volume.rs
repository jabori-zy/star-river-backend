use crate::calculate_fn;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::indicator_engine::talib::TALib;
use star_river_core::cache::CacheValue;
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::indicator_define::volume::*;
use star_river_core::market::Kline;
use std::sync::Arc;

impl CalculateIndicatorFunction {
    // AD - Chaikin A/D Line #钱德动量线
    calculate_fn!(AD,
        input => [high,low,close,volume]
    );

    // ADOSC - Chaikin A/D Oscillator #钱德动量振荡器
    calculate_fn!(ADOSC,
        input => [high,low,close,volume],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
        ]
    );

    // OBV - On Balance Volume #能量潮
    calculate_fn!(OBV,
        input => [close, volume]
    );
}
