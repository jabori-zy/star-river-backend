use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::indicator_engine::talib::TALib;
use crate::{calculate_fn, calculate_fn_snake};
use star_river_core::cache::CacheValue;
use star_river_core::indicator::indicator_define::overlap::*;
use star_river_core::indicator::Indicator;
use std::sync::Arc;

impl CalculateIndicatorFunction {
    // Bollinger Bands #�&
    calculate_fn!(BBANDS,
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: MAType),
        ]
    );

    // Double Exponential Moving Average #�p��sG�
    calculate_fn!(DEMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // Exponential Moving Average #p��sG�
    calculate_fn!(EMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // Hilbert Transform - Instantaneous Trendline #/y�����
    calculate_fn_snake!(HtTrendline,
        talib_params => []
    );

    // KAMA - Kaufman Adaptive Moving Average #a+�����sG�
    calculate_fn!(KAMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // MA
    calculate_fn!(MA,
        talib_params => [
            (time_period: i32),
            (ma_type: MAType),
        ]
    );

    // MAMA - MESA Adaptive Moving Average #�(����sG�
    calculate_fn!(MAMA,
        talib_params => [
            (fast_limit: f64),
            (slow_limit: f64),
        ]
    );

    // MAVP - Moving average with variable period #��sG�h
    // calculate_fn!(MAVP,
    //     input => [close],
    //     talib_params => [
    //         (min_period: i32),
    //         (max_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // MIDPOINT - MidPoint over period #h-�
    calculate_fn!(MIDPOINT,
        talib_params => [
            (time_period: i32),
        ]
    );

    // MIDPRICE - Midpoint Price over period #h-��<
    calculate_fn!(MIDPRICE,
        input => [high, low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // SAR - Parabolic SAR #�i�l
    calculate_fn!(SAR,
        input => [high, low],
        talib_params => [
            (acceleration: f64),
            (maximum: f64),
        ]
    );

    // SAREXT - Parabolic SAR - Extended #�i�liU
    calculate_fn!(SAREXT,
        input => [high, low],
        talib_params => [
            (start_value: f64),
            (offset_on_reverse: f64),
            (acceleration_init_long: f64),
            (acceleration_long: f64),
            (acceleration_max_long: f64),
            (acceleration_init_short: f64),
            (acceleration_short: f64),
            (acceleration_max_short: f64),
        ]
    );

    // SMA - Simple Moving Average #�U��sG�
    calculate_fn!(SMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // T3 - Triple Exponential Moving Average (T3) #	�p��sG�
    calculate_fn!(T3,
        talib_params => [
            (time_period: i32),
            (v_factor: f64),
        ]
    );

    // TEMA - Triple Exponential Moving Average #	�p��sG�
    calculate_fn!(TEMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // TRIMA - Triangular Moving Average #	�b��sG�
    calculate_fn!(TRIMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // WMA - Weighted Moving Average #�C��sG�
    calculate_fn!(WMA,
        talib_params => [
            (time_period: i32),
        ]
    );
}
