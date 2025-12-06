use ta_lib::{Indicator, indicator::overlap::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn, calculate_fn_snake};

impl CalculateIndicatorFunction {
    // Bollinger Bands
    calculate_fn!(BBANDS,
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: MAType),
        ]
    );

    // Double Exponential Moving Average
    calculate_fn!(DEMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // Exponential Moving Average
    calculate_fn!(EMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // Hilbert Transform - Instantaneous Trendline
    calculate_fn_snake!(HtTrendline,
        talib_params => []
    );

    // KAMA - Kaufman Adaptive Moving Average
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

    // MAMA - MESA Adaptive Moving Average
    calculate_fn!(MAMA,
        talib_params => [
            (fast_limit: f64),
            (slow_limit: f64),
        ]
    );

    // MAVP - Moving average with variable period
    // calculate_fn!(MAVP,
    //     input => [close],
    //     talib_params => [
    //         (min_period: i32),
    //         (max_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // MIDPOINT - MidPoint over period
    calculate_fn!(MIDPOINT,
        talib_params => [
            (time_period: i32),
        ]
    );

    // MIDPRICE - Midpoint Price over period
    calculate_fn!(MIDPRICE,
        input => [high, low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // SAR - Parabolic SAR
    calculate_fn!(SAR,
        input => [high, low],
        talib_params => [
            (acceleration: f64),
            (maximum: f64),
        ]
    );

    // SAREXT - Parabolic SAR - Extended
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

    // SMA - Simple Moving Average
    calculate_fn!(SMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // T3 - Triple Exponential Moving Average (T3)
    calculate_fn!(T3,
        talib_params => [
            (time_period: i32),
            (v_factor: f64),
        ]
    );

    // TEMA - Triple Exponential Moving Average
    calculate_fn!(TEMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // TRIMA - Triangular Moving Average
    calculate_fn!(TRIMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // WMA - Weighted Moving Average
    calculate_fn!(WMA,
        talib_params => [
            (time_period: i32),
        ]
    );
}
