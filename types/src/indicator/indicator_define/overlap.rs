use crate::{define_indicator, indicator::{MAType, PriceSource}};

//Bollinger Bands #布林带
define_indicator!(BBands,
    params => [(time_period: i32), (dev_up: f64), (dev_down: f64), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (upper: f64), (middle: f64), (lower: f64)],
);


//Double Exponential Moving Average #双指数移动平均线
define_indicator!(DEMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (dema: f64)],
);

//Exponential Moving Average #指数移动平均线
define_indicator!(EMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (ema: f64)],
);


//Hilbert Transform - Instantaneous Trendline #希尔伯特瞬时趋势线
define_indicator!(HtTrendline,
    params => [(price_source: PriceSource)],
    output => [(timestamp: i64), (ht_trendline: f64)],
);

//KAMA                 Kaufman Adaptive Moving Average #卡夫曼自适应移动平均线
define_indicator!(KAMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (kama: f64)],
);

//MA
define_indicator!(MA,
    params => [(time_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (ma: f64)],
);


//MAMA                 MESA Adaptive Moving Average #梅萨自适应移动平均线 
define_indicator!(MAMA,
    params => [(fast_limit: f64), (slow_limit: f64), (price_source: PriceSource)],
    output => [(timestamp: i64), (mama: f64), (fama: f64)],
);

//MAVP                 Moving average with variable period #移动平均变周期
// define_indicator!(MAVP,
//     params => [(min_period: i32), (max_period: i32), (ma_type: MAType), (price_source: PriceSource)],
//     output => [(timestamp: i64), (mavp: f64)],
// );

//MIDPOINT             MidPoint over period #周期中点
define_indicator!(MIDPOINT,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (midpoint: f64)],
);

//MIDPRICE             Midpoint Price over period #周期中点价格
define_indicator!(MIDPRICE,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (midprice: f64)],
);

//SAR                  Parabolic SAR #抛物线转向
define_indicator!(SAR,
    params => [(acceleration: f64), (maximum: f64)],
    output => [(timestamp: i64), (sar: f64)],
);

//SAREXT               Parabolic SAR - Extended #抛物线转向扩展
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
    output => [(timestamp: i64), (sarext: f64)],
);

//SMA                  Simple Moving Average #简单移动平均线
define_indicator!(SMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (sma: f64)],
);

//T3                   Triple Exponential Moving Average (T3) #三重指数移动平均线
define_indicator!(T3,
    params => [(time_period: i32), (v_factor: f64), (price_source: PriceSource)],
    output => [(timestamp: i64), (t3: f64)],
);

//TEMA                 Triple Exponential Moving Average #三重指数移动平均线
define_indicator!(TEMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (tema: f64)],
);

//TRIMA                Triangular Moving Average #三角形移动平均线
define_indicator!(TRIMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (trima: f64)],
);

//WMA                  Weighted Moving Average #加权移动平均线
define_indicator!(WMA,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (wma: f64)],
);





