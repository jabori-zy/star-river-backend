use crate::calculate_fn;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::indicator_engine::talib::TALib;
use star_river_core::cache::CacheValue;
use star_river_core::indicator::indicator_define::volatility::*;
use star_river_core::indicator::Indicator;
use std::sync::Arc;
use star_river_core::error::engine_error::indicator_engine_error::*;

impl CalculateIndicatorFunction {
    // ATR - Average True Range #平均真实波幅
    calculate_fn!(ATR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // NATR - Normalized Average True Range #归一化平均真实波幅
    calculate_fn!(NATR,
        input => [high, low, close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // TRANGE - True Range #真实波幅
    calculate_fn!(TRANGE,
        input => [high, low, close]
    );
}
