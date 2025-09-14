use chrono::{DateTime, Utc};

use crate::{
    define_indicator,
    indicator::{MAType, PriceSource},
};

//ADX  Average Directional Movement Index #平均方向性指数
define_indicator!(ADX,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (adx: f64)],
);

// ADXR  Average Directional Movement Index Rating #平均方向性指数评级
define_indicator!(ADXR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (adxr: f64)],
);

// APO  Absolute Price Oscillator #绝对价格振荡器
define_indicator!(APO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (apo: f64)],
);

// AROON           Aroon #阿隆指标
define_indicator!(AROON,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (aroon_down: f64), (aroon_up: f64)],
);

// AROONOSC             Aroon Oscillator #阿隆振荡器
define_indicator!(AROONOSC,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (aroonosc: f64)],
);

// BOP                  Balance Of Power #平衡力量
define_indicator!(BOP,
    params => [],
    output => [(datetime: DateTime<Utc>), (bop: f64)],
);

// CCI           Commodity Channel Index #商品通道指数
define_indicator!(CCI,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (cci: f64)],
);

// CMO                  Chande Momentum Oscillator
define_indicator!(CMO,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (cmo: f64)],
);

// DX                   Directional Movement Index #方向性指数
define_indicator!(DX,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (dx: f64)],
);

// MACD                 Moving Average Convergence/Divergence
define_indicator!(MACD,
    params => [(fast_period: i32), (slow_period: i32), (signal_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (macd: f64), (signal: f64), (histogram: f64)],
);

// MACDEXT              MACD with controllable MA type #可控MA类型的MACD
define_indicator!(MACDEXT,
    params => [(fast_period: i32), (fast_ma_type: MAType), (slow_period: i32), (slow_ma_type: MAType), (signal_period: i32), (signal_ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (macd: f64), (signal: f64), (histogram: f64)],
);

// MACDFIX              Moving Average Convergence/Divergence Fix 12/26 #12/26固定MACD
define_indicator!(MACDFIX,
    params => [(signal_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (macd: f64), (signal: f64), (histogram: f64)],
);

// MFI                  Money Flow Index #资金流量指数
define_indicator!(MFI,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (mfi: f64)],
);

// MINUS_DI             Minus Directional Indicator #负方向性指标
define_indicator!(MinusDi,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (minus_di: f64)],
);

// MINUS_DM             Minus Directional Movement #负方向性运动
define_indicator!(MinusDm,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (minus_dm: f64)],
);

// MOM                  Momentum #动量
define_indicator!(MOM,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (mom: f64)],
);

// PLUS_DI              Plus Directional Indicator
define_indicator!(PlusDi,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (plus_di: f64)],
);

// PLUS_DM              Plus Directional Movement
define_indicator!(PlusDm,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (plus_dm: f64)],
);

// PPO                  Percentage Price Oscillator
define_indicator!(PPO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (ppo: f64)],
);

// ROC                  Rate of change : ((price/prevPrice)-1)*100 #变化率
define_indicator!(ROC,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (roc: f64)],
);

// ROCP                 Rate of change Percentage: (price-prevPrice)/prevPrice #变化率百分比
define_indicator!(ROCP,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rocp: f64)],
);

// ROCR                 Rate of change ratio: (price/prevPrice) #变化率比率
define_indicator!(ROCR,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rocr: f64)],
);

// ROCR100              Rate of change ratio 100 scale: (price/prevPrice)*100 #变化率比率100比例
define_indicator!(ROCR100,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rocr100: f64)],
);

// RSI                  Relative Strength Index #相对强弱指数
define_indicator!(RSI,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (rsi: f64)],
);

// STOCH                Stochastic #随机指标
define_indicator!(STOCH,
    params => [(fast_k_period: i32), (slow_k_period: i32), (slow_k_ma_type: MAType), (slow_d_period: i32), (slow_d_ma_type: MAType)],
    output => [(datetime: DateTime<Utc>), (slow_k: f64), (slow_d: f64)],
);

// STOCHF               Stochastic Fast #快速随机指标
define_indicator!(STOCHF,
    params => [(fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MAType)],
    output => [(datetime: DateTime<Utc>), (fast_k: f64), (fast_d: f64)],
);

// STOCHRSI             Stochastic Relative Strength Index #随机相对强弱指数
define_indicator!(STOCHRSI,
    params => [(time_period: i32), (fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MAType), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (fast_k: f64), (fast_d: f64)],
);

// TRIX                 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
define_indicator!(TRIX,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(datetime: DateTime<Utc>), (trix: f64)],
);

// ULTOSC               Ultimate Oscillator #终极振荡器
define_indicator!(ULTOSC,
    params => [(time_period1: i32), (time_period2: i32), (time_period3: i32)],
    output => [(datetime: DateTime<Utc>), (ultosc: f64)],
);

// WILLR                Williams' %R #威廉指标
define_indicator!(WILLR,
    params => [(time_period: i32)],
    output => [(datetime: DateTime<Utc>), (willr: f64)],
);
