use chrono::{DateTime, Utc};

use crate::{
    define_indicator,
    indicator::{MAType, PriceSource},
};

//Bollinger Bands
define_indicator!(BBANDS,
    params => [(time_period: i32), (dev_up: f64), (dev_down: f64), (ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (upper: Option<f64>), (middle: Option<f64>), (lower: Option<f64>)],
);

//Double Exponential Moving Average
define_indicator!(DEMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (dema: Option<f64>)],
);

//Exponential Moving Average
define_indicator!(EMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ema: Option<f64>)],
);

//Hilbert Transform - Instantaneous Trendline
define_indicator!(HtTrendline,
    params => [(price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ht_trendline: Option<f64>)],
);

//KAMA                 Kaufman Adaptive Moving Average
define_indicator!(KAMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (kama: Option<f64>)],
);

//MA
define_indicator!(MA,
    params => [(time_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ma: Option<f64>)],
);

//MAMA                 MESA Adaptive Moving Average
define_indicator!(MAMA,
    params => [(fast_limit: f64), (slow_limit: f64), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (mama: Option<f64>), (fama: Option<f64>)],
);

//MAVP                 Moving average with variable period
// define_indicator!(MAVP,
//     params => [(min_period: i32), (max_period: i32), (ma_type: MAType), (price_source: PriceSource)],
//     output => [(datetime: DateTime<Utc>), (mavp: f64)],
// );

//MIDPOINT             MidPoint over period
define_indicator!(MIDPOINT,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (midpoint: Option<f64>)],
);

//MIDPRICE             Midpoint Price over period
define_indicator!(MIDPRICE,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (midprice: Option<f64>)],
);

//SAR                  Parabolic SAR
define_indicator!(SAR,
    params => [(acceleration: f64), (maximum: f64)],
    output => [(datetime: DateTime<Utc>), (sar: Option<f64>)],
);

//SAREXT               Parabolic SAR - Extended
define_indicator!(SAREXT,
    params => [
        (start_value: f64),
        (offset_on_reverse: f64),
        (acceleration_init_long: f64),
        (acceleration_long: f64),
        (acceleration_max_long: f64),
        (acceleration_init_short: f64),
        (acceleration_short: f64),
        (acceleration_max_short: f64)],
    output => [(datetime: DateTime<Utc>), (sarext: Option<f64>)],
);

//SMA                  Simple Moving Average
define_indicator!(SMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (sma: Option<f64>)],
);

//T3                   Triple Exponential Moving Average (T3)
define_indicator!(T3,
    params => [(time_period: i32), (v_factor: f64), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (t3: Option<f64>)],
);

//TEMA                 Triple Exponential Moving Average
define_indicator!(TEMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (tema: Option<f64>)],
);

//TRIMA                Triangular Moving Average
define_indicator!(TRIMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (trima: Option<f64>)],
);

//WMA                  Weighted Moving Average
define_indicator!(WMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (wma: Option<f64>)],
);
