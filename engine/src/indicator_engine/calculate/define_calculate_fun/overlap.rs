use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::indicator_engine::talib::TALib;
use crate::{calculate_fn, calculate_fn_snake};
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::market::Kline;
use star_river_core::indicator::indicator_define::overlap::*;
use std::sync::Arc;

impl CalculateIndicatorFunction {
    // Bollinger Bands #布林带
    calculate_fn!(BBANDS,
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: MAType),
        ]
    );

    // Double Exponential Moving Average #双指数移动平均线
    calculate_fn!(DEMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // Exponential Moving Average #指数移动平均线
    calculate_fn!(EMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // Hilbert Transform - Instantaneous Trendline #希尔伯特变换瞬时趋势线
    calculate_fn_snake!(HtTrendline,
        talib_params => []
    );

    // KAMA - Kaufman Adaptive Moving Average #考夫曼自适应移动平均线
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

    // MAMA - MESA Adaptive Moving Average #MESA自适应移动平均线
    calculate_fn!(MAMA,
        talib_params => [
            (fast_limit: f64),
            (slow_limit: f64),
        ]
    );

    // MAVP - Moving average with variable period #可变周期移动平均线
    // calculate_fn!(MAVP,
    //     input => [close],
    //     talib_params => [
    //         (min_period: i32),
    //         (max_period: i32),
    //         (ma_type: MAType),
    //     ]
    // );

    // MIDPOINT - MidPoint over period #周期中点价
    calculate_fn!(MIDPOINT,
        talib_params => [
            (time_period: i32),
        ]
    );

    // MIDPRICE - Midpoint Price over period #周期中点价格
    calculate_fn!(MIDPRICE,
        input => [high, low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // SAR - Parabolic SAR #抛物线转向
    calculate_fn!(SAR,
        input => [high, low],
        talib_params => [
            (acceleration: f64),
            (maximum: f64),
        ]
    );

    // SAREXT - Parabolic SAR - Extended #抛物线转向扩展版
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

    // SMA - Simple Moving Average #简单移动平均线
    calculate_fn!(SMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // T3 - Triple Exponential Moving Average (T3) #三重指数移动平均线
    calculate_fn!(T3,
        talib_params => [
            (time_period: i32),
            (v_factor: f64),
        ]
    );

    // TEMA - Triple Exponential Moving Average #三重指数移动平均线
    calculate_fn!(TEMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // TRIMA - Triangular Moving Average #三角移动平均线
    calculate_fn!(TRIMA,
        talib_params => [
            (time_period: i32),
        ]
    );

    // WMA - Weighted Moving Average #加权移动平均线
    calculate_fn!(WMA,
        talib_params => [
            (time_period: i32),
        ]
    );
}
