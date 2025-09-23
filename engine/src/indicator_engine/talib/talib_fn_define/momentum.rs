use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::talib_bindings::*;
// use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use crate::talib_snake_fn;
use chrono::{DateTime, Utc};
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::indicator_define::momentum::*;

impl TALib {
    //ADX  Average Directional Movement Index #平均方向性指数
    talib_fn!(
        ADX,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(adx: f64)],
    );

    // ADXR  Average Directional Movement Index Rating #平均方向性指数评级
    talib_fn!(
        ADXR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(adxr: f64)],
    );

    // APO  Absolute Price Oscillator #绝对价格振荡器
    talib_fn!(
        APO,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32), (ma_type: i32)],
        output => [(apo: f64)],
    );

    // AROON           Aroon #阿隆指标
    talib_fn!(
        AROON,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(aroon_down: f64), (aroon_up: f64)],
    );

    // AROONOSC             Aroon Oscillator #阿隆振荡器
    talib_fn!(
        AROONOSC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(aroonosc: f64)],
    );

    // BOP                  Balance Of Power #平衡力量
    talib_fn!(
        BOP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(bop: f64)],
    );

    // CCI           Commodity Channel Index #商品通道指数
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

    // DX                   Directional Movement Index #方向性指数
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

    // MACDEXT              MACD with controllable MA type #可控MA类型的MACD
    talib_fn!(
        MACDEXT,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(fast_period: i32), (fast_ma_type: i32), (slow_period: i32), (slow_ma_type: i32), (signal_period: i32), (signal_ma_type: i32)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    // MACDFIX              Moving Average Convergence/Divergence Fix 12/26 #12/26固定MACD
    talib_fn!(
        MACDFIX,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(signal_period: i32)],
        output => [(macd: f64), (signal: f64), (histogram: f64)],
    );

    // MFI                  Money Flow Index #资金流量指数
    talib_fn!(
        MFI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(mfi: f64)],
    );

    // MINUS_DI             Minus Directional Indicator #负方向性指标
    talib_snake_fn!(
        MinusDi,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(minus_di: f64)],
    );

    // MINUS_DM             Minus Directional Movement #负方向性运动
    talib_snake_fn!(
        MinusDm,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(minus_dm: f64)],
    );

    // MOM                  Momentum #动量
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
        talib_params => [(fast_period: i32), (slow_period: i32), (ma_type: i32)],
        output => [(ppo: f64)],
    );

    // ROC                  Rate of change : ((price/prevPrice)-1)*100 #变化率
    talib_fn!(
        ROC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(roc: f64)],
    );

    // ROCP                 Rate of change Percentage: (price-prevPrice)/prevPrice #变化率百分比
    talib_fn!(
        ROCP,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rocp: f64)],
    );

    // ROCR                 Rate of change ratio: (price/prevPrice) #变化率比率
    talib_fn!(
        ROCR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rocr: f64)],
    );

    // ROCR100              Rate of change ratio 100 scale: (price/prevPrice)*100 #变化率比率100比例
    talib_fn!(
        ROCR100,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rocr100: f64)],
    );

    // RSI                  Relative Strength Index #相对强弱指数
    talib_fn!(
        RSI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(rsi: f64)],
    );

    // STOCH                Stochastic #随机指标
    talib_fn!(
        STOCH,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(fast_k_period: i32), (slow_k_period: i32), (slow_k_ma_type: i32), (slow_d_period: i32), (slow_d_ma_type: i32)],
        output => [(slow_k: f64), (slow_d: f64)],
    );

    // STOCHF               Stochastic Fast #快速随机指标
    talib_fn!(
        STOCHF,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: i32)],
        output => [(fast_k: f64), (fast_d: f64)],
    );

    // STOCHRSI             Stochastic Relative Strength Index #随机相对强弱指数
    talib_fn!(
        STOCHRSI,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [(time_period: i32), (fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: i32)],
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

    // ULTOSC               Ultimate Oscillator #终极振荡器
    talib_fn!(
        ULTOSC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period1: i32), (time_period2: i32), (time_period3: i32)],
        output => [(ultosc: f64)],
    );

    // WILLR                Williams' %R #威廉指标
    talib_fn!(
        WILLR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(willr: f64)],
    );
}
