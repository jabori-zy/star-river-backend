use chrono::{DateTime, Utc};

// use crate::indicator_engine::talib_error::TalibError;
use super::MaTypeInt;
use crate::{Indicator, indicator::momentum::*, talib::TALib, talib_bindings::*, talib_fn, talib_snake_fn};

impl TALib {
    // ADX - Average Directional Movement Index
    talib_fn!(
        ADX,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(adx: f64)],
    );

    // ADXR - Average Directional Movement Index Rating
    talib_fn!(
        ADXR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(adxr: f64)],
    );

    // APO - Absolute Price Oscillator
    talib_fn!(
        APO,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32), (ma_type: MaTypeInt)],
        output => [(apo: f64)],
    );

    // AROON - Aroon
    talib_fn!(
        AROON,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(aroon_down: f64), (aroon_up: f64)],
    );

    // AROONOSC - Aroon Oscillator
    talib_fn!(
        AROONOSC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(aroonosc: f64)],
    );

    // BOP - Balance Of Power
    talib_fn!(
        BOP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(bop: f64)],
    );

    // CCI - Commodity Channel Index
    talib_fn!(
        CCI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(cci: f64)],
    );

    // CMO                  Chande Momentum Oscillator
    talib_fn!(
        CMO,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(cmo: f64)],
    );

    // DX - Directional Movement Index
    talib_fn!(
        DX,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(dx: f64)],
    );

    // MACD                 Moving Average Convergence/Divergence
    talib_fn!(
        MACD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32), (signal_period: i32)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    // MACDEXT - MACD with controllable MA type
    talib_fn!(
        MACDEXT,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (fast_ma_type: MaTypeInt), (slow_period: i32), (slow_ma_type: MaTypeInt), (signal_period: i32), (signal_ma_type: MaTypeInt)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    // MACDFIX - Moving Average Convergence/Divergence Fix 12/26
    talib_fn!(
        MACDFIX,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(signal_period: i32)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    // MFI - Money Flow Index
    talib_fn!(
        MFI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(mfi: f64)],
    );

    // MINUS_DI - Minus Directional Indicator
    talib_snake_fn!(
        MinusDi,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(minus_di: f64)],
    );

    // MINUS_DM - Minus Directional Movement
    talib_snake_fn!(
        MinusDm,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(minus_dm: f64)],
    );

    // MOM - Momentum
    talib_fn!(
        MOM,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(mom: f64)],
    );

    // PLUS_DI              Plus Directional Indicator
    talib_snake_fn!(
        PlusDi,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(plus_di: f64)],
    );

    // PLUS_DM              Plus Directional Movement
    talib_snake_fn!(
        PlusDm,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(plus_dm: f64)],
    );

    // PPO                  Percentage Price Oscillator
    talib_fn!(
        PPO,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32), (ma_type: MaTypeInt)],
        output => [(ppo: f64)],
    );

    // ROC - Rate of change: ((price/prevPrice)-1)*100
    talib_fn!(
        ROC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(roc: f64)],
    );

    // ROCP - Rate of change Percentage: (price-prevPrice)/prevPrice
    talib_fn!(
        ROCP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rocp: f64)],
    );

    // ROCR - Rate of change ratio: (price/prevPrice)
    talib_fn!(
        ROCR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rocr: f64)],
    );

    // ROCR100 - Rate of change ratio 100 scale: (price/prevPrice)*100
    talib_fn!(
        ROCR100,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rocr100: f64)],
    );

    // RSI - Relative Strength Index
    talib_fn!(
        RSI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rsi: f64)],
    );

    // STOCH - Stochastic
    talib_fn!(
        STOCH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(fast_k_period: i32), (slow_k_period: i32), (slow_k_ma_type: MaTypeInt), (slow_d_period: i32), (slow_d_ma_type: MaTypeInt)],
        output => [(slow_k: f64), (slow_d: f64)],
    );

    // STOCHF - Stochastic Fast
    talib_fn!(
        STOCHF,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MaTypeInt)],
        output => [(fast_k: f64), (fast_d: f64)],
    );

    // STOCHRSI - Stochastic Relative Strength Index
    talib_fn!(
        STOCHRSI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32), (fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MaTypeInt)],
        output => [(fast_k: f64), (fast_d: f64)],
    );

    // TRIX                 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
    talib_fn!(
        TRIX,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(trix: f64)],
    );

    // ULTOSC - Ultimate Oscillator
    talib_fn!(
        ULTOSC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period1: i32), (time_period2: i32), (time_period3: i32)],
        output => [(ultosc: f64)],
    );

    // WILLR - Williams' %R
    talib_fn!(
        WILLR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(willr: f64)],
    );
}
