use chrono::{DateTime, Utc};

use crate::{
    define_indicator,
    indicator::{MAType, PriceSource},
};

//ADX  Average Directional Movement Index
define_indicator!(ADX,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (adx: Option<f64>)],
);

// ADXR  Average Directional Movement Index Rating
define_indicator!(ADXR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (adxr: Option<f64>)],
);

// APO  Absolute Price Oscillator
define_indicator!(APO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (apo: Option<f64>)],
);

// AROON           Aroon
define_indicator!(AROON,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (aroon_down: Option<f64>), (aroon_up: Option<f64>)],
);

// AROONOSC             Aroon Oscillator
define_indicator!(AROONOSC,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (aroonosc: Option<f64>)],
);

// BOP                  Balance Of Power
define_indicator!(BOP,
    params => [],
    output => [(datetime: DateTime<Utc>), (bop: Option<f64>)],
);

// CCI           Commodity Channel Index
define_indicator!(CCI,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (cci: Option<f64>)],
);

// CMO                  Chande Momentum Oscillator
define_indicator!(CMO,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (cmo: Option<f64>)],
);

// DX                   Directional Movement Index
define_indicator!(DX,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (dx: Option<f64>)],
);

// MACD                 Moving Average Convergence/Divergence
define_indicator!(MACD,
    params => [(fast_period: i32), (slow_period: i32), (signal_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (macd: Option<f64>), (signal: Option<f64>), (histogram: Option<f64>)],
);

// MACDEXT              MACD with controllable MA type
define_indicator!(MACDEXT,
    params => [(fast_period: i32), (fast_ma_type: MAType), (slow_period: i32), (slow_ma_type: MAType), (signal_period: i32), (signal_ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (macd: Option<f64>), (signal: Option<f64>), (histogram: Option<f64>)],
);

// MACDFIX              Moving Average Convergence/Divergence Fix 12/26
define_indicator!(MACDFIX,
    params => [(signal_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (macd: Option<f64>), (signal: Option<f64>), (histogram: Option<f64>)],
);

// MFI                  Money Flow Index
define_indicator!(MFI,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (mfi: Option<f64>)],
);

// MINUS_DI             Minus Directional Indicator
define_indicator!(MinusDi,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (minus_di: Option<f64>)],
);

// MINUS_DM             Minus Directional Movement
define_indicator!(MinusDm,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (minus_dm: Option<f64>)],
);

// MOM                  Momentum
define_indicator!(MOM,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (mom: Option<f64>)],
);

// PLUS_DI              Plus Directional Indicator
define_indicator!(PlusDi,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (plus_di: Option<f64>)],
);

// PLUS_DM              Plus Directional Movement
define_indicator!(PlusDm,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (plus_dm: Option<f64>)],
);

// PPO                  Percentage Price Oscillator
define_indicator!(PPO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ppo: Option<f64>)],
);

// ROC                  Rate of change : ((price/prevPrice)-1)*100
define_indicator!(ROC,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (roc: Option<f64>)],
);

// ROCP                 Rate of change Percentage: (price-prevPrice)/prevPrice
define_indicator!(ROCP,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rocp: Option<f64>)],
);

// ROCR                 Rate of change ratio: (price/prevPrice)
define_indicator!(ROCR,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rocr: Option<f64>)],
);

// ROCR100              Rate of change ratio 100 scale: (price/prevPrice)*100
define_indicator!(ROCR100,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rocr100: Option<f64>)],
);

// RSI                  Relative Strength Index
define_indicator!(RSI,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rsi: Option<f64>)],
);

// STOCH                Stochastic
define_indicator!(STOCH,
    params => [(fast_k_period: i32), (slow_k_period: i32), (slow_k_ma_type: MAType), (slow_d_period: i32), (slow_d_ma_type: MAType)],
    output => [(datetime: DateTime<Utc>), (slow_k: Option<f64>), (slow_d: Option<f64>)],
);

// STOCHF               Stochastic Fast
define_indicator!(STOCHF,
    params => [(fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MAType)],
    output => [(datetime: DateTime<Utc>), (fast_k: Option<f64>), (fast_d: Option<f64>)],
);

// STOCHRSI             Stochastic Relative Strength Index
define_indicator!(STOCHRSI,
    params => [(time_period: i32), (fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (fast_k: Option<f64>), (fast_d: Option<f64>)],
);

// TRIX                 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
define_indicator!(TRIX,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (trix: Option<f64>)],
);

// ULTOSC               Ultimate Oscillator
define_indicator!(ULTOSC,
    params => [(time_period1: i32), (time_period2: i32), (time_period3: i32)],
    output => [(datetime: DateTime<Utc>), (ultosc: Option<f64>)],
);

// WILLR                Williams' %R
define_indicator!(WILLR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (willr: Option<f64>)],
);
