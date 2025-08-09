use crate::{define_indicator, indicator::{MAType, PriceSource}};


//ADX  Average Directional Movement Index #平均方向性指数
define_indicator!(ADX,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (adx: f64)],
);


// ADXR  Average Directional Movement Index Rating #平均方向性指数评级
define_indicator!(ADXR,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (adxr: f64)],
);


// APO  Absolute Price Oscillator #绝对价格振荡器
define_indicator!(APO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (apo: f64)],
);


// AROON           Aroon #阿隆指标
define_indicator!(AROON,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (aroon_down: f64), (aroon_up: f64)],
);



// AROONOSC             Aroon Oscillator #阿隆振荡器
define_indicator!(AROONOSC,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (aroonosc: f64)],
);


// BOP                  Balance Of Power #平衡力量
define_indicator!(BOP,
    params => [],
    output => [(timestamp: i64), (bop: f64)],
);


// CCI           Commodity Channel Index #商品通道指数
define_indicator!(CCI,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (cci: f64)],
);



// CMO                  Chande Momentum Oscillator
define_indicator!(CMO,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (cmo: f64)],
);



// DX                   Directional Movement Index #方向性指数
define_indicator!(DX,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (dx: f64)],
);


 
// MACD                 Moving Average Convergence/Divergence
define_indicator!(MACD,
    params => [(fast_period: i32), (slow_period: i32), (signal_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);

// MACDEXT              MACD with controllable MA type #可控MA类型的MACD
define_indicator!(MACDEXT,
    params => [(fast_period: i32), (fast_ma_type: MAType), (slow_period: i32), (slow_ma_type: MAType), (signal_period: i32), (signal_ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);



// MACDFIX              Moving Average Convergence/Divergence Fix 12/26 #12/26固定MACD
define_indicator!(MACDFIX,
    params => [(signal_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);


// MFI                  Money Flow Index #资金流量指数
define_indicator!(MFI,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (mfi: f64)],
);


// MINUS_DI             Minus Directional Indicator #负方向性指标
define_indicator!(MinusDi,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (minus_di: f64)],
);


// MINUS_DM             Minus Directional Movement #负方向性运动
define_indicator!(MinusDm,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (minus_dm: f64)],
);


// MOM                  Momentum #动量
define_indicator!(MOM,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (mom: f64)],
);


// PLUS_DI              Plus Directional Indicator
define_indicator!(PlusDi,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (plus_di: f64)],
);


// PLUS_DM              Plus Directional Movement
define_indicator!(PlusDm,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (plus_dm: f64)],
);


// PPO                  Percentage Price Oscillator
define_indicator!(PPO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (ppo: f64)],
);


// ROC                  Rate of change : ((price/prevPrice)-1)*100 #变化率
define_indicator!(ROC,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (roc: f64)],
);

// ROCP                 Rate of change Percentage: (price-prevPrice)/prevPrice #变化率百分比
define_indicator!(ROCP,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rocp: f64)],
);


// ROCR                 Rate of change ratio: (price/prevPrice) #变化率比率
define_indicator!(ROCR,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rocr: f64)],
);



// ROCR100              Rate of change ratio 100 scale: (price/prevPrice)*100 #变化率比率100比例
define_indicator!(ROCR100,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rocr100: f64)],
);


// RSI                  Relative Strength Index #相对强弱指数
define_indicator!(RSI,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rsi: f64)],
);



// STOCH                Stochastic #随机指标
define_indicator!(STOCH,
    params => [(fast_k_period: i32), (slow_k_period: i32), (slow_k_ma_type: MAType), (slow_d_period: i32), (slow_d_ma_type: MAType)],
    output => [(timestamp: i64), (slow_k: f64), (slow_d: f64)],
);


// STOCHF               Stochastic Fast #快速随机指标
define_indicator!(STOCHF,
    params => [(fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MAType)],
    output => [(timestamp: i64), (fast_k: f64), (fast_d: f64)],
);


// STOCHRSI             Stochastic Relative Strength Index #随机相对强弱指数
define_indicator!(STOCHRSI,
    params => [(time_period: i32), (fast_k_period: i32), (fast_d_period: i32), (fast_d_ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (fast_k: f64), (fast_d: f64)],
);


// TRIX                 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
define_indicator!(TRIX,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (trix: f64)],
);


// ULTOSC               Ultimate Oscillator #终极振荡器
define_indicator!(ULTOSC,
    params => [(time_period1: i32), (time_period2: i32), (time_period3: i32)],
    output => [(timestamp: i64), (ultosc: f64)],
);


// WILLR                Williams' %R #威廉指标
define_indicator!(WILLR,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (willr: f64)],
);


