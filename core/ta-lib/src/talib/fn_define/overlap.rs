use chrono::{DateTime, Utc};

// use crate::indicator_engine::talib_error::TalibError;
use super::MaTypeInt;
use crate::{Indicator, indicator::overlap::*, talib::TALib, talib_bindings::*, talib_fn, talib_snake_fn};

impl TALib {
    //Bollinger Bands
    talib_fn!(
        BBANDS,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: MaTypeInt),
        ],
        output => [(upper: f64), (middle: f64), (lower: f64)],
    );

    //Double Exponential Moving Average
    talib_fn!(
        DEMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(dema: f64)],
    );

    //Exponential Moving Average
    talib_fn!(
        EMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(ema: f64)],
    );

    //Hilbert Transform - Instantaneous Trendline
    talib_snake_fn!(
        HtTrendline,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [],
        output => [(ht_trendline: f64)],
    );

    //KAMA                 Kaufman Adaptive Moving Average
    talib_fn!(
        KAMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(kama: f64)],
    );

    //MA
    talib_fn!(
        MA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32),
            (ma_type: MaTypeInt),
        ],
        output => [(ma: f64)],
    );

    //MAMA                 MESA Adaptive Moving Average
    talib_fn!(
        MAMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_limit: f64), (slow_limit: f64)],
        output => [(mama: f64), (fama: f64)],
    );

    //MAVP                 Moving average with variable period
    // talib_fn!(
    //     MAVP,
    //     datetime => (datetime_list: &[DateTime<Utc>]),
    //     input => [(data: &[f64]), (periods: &[f64])],
    //     talib_params => [(min_period: i32), (max_period: i32), (ma_type: i32)],
    //     output => [(mavp: f64)],
    // );

    //MIDPOINT             MidPoint over period
    talib_fn!(
        MIDPOINT,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(midpoint: f64)],
    );

    //MIDPRICE             Midpoint Price over period
    talib_fn!(
        MIDPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(midprice: f64)],
    );

    //SAR                  Parabolic SAR
    talib_fn!(
        SAR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(acceleration: f64), (maximum: f64)],
        output => [(sar: f64)],
    );

    //SAREXT               Parabolic SAR - Extended
    talib_fn!(
        SAREXT,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [
            (start_value: f64),
            (offset_on_reverse: f64),
            (acceleration_init_long: f64),
            (acceleration_long: f64),
            (acceleration_max_long: f64),
            (acceleration_init_short: f64),
            (acceleration_short: f64),
            (acceleration_max_short: f64)
        ],
        output => [(sarext: f64)],
    );

    //SMA                  Simple Moving Average
    talib_fn!(
        SMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(sma: f64)],
    );

    //T3                   Triple Exponential Moving Average (T3)
    talib_fn!(
        T3,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32), (v_factor: f64)],
        output => [(t3: f64)],
    );

    //TEMA                 Triple Exponential Moving Average
    talib_fn!(
        TEMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(tema: f64)],
    );

    //TRIMA                Triangular Moving Average
    talib_fn!(
        TRIMA,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(trima: f64)],
    );

    //WMA                  Weighted Moving Average
    talib_fn!(
    WMA,
    datetime => (datetime_list: &[DateTime<Utc>]),
    input => [(data: &[f64])],
    talib_params => [(time_period: i32)],
        output => [(wma: f64)],
    );
}
