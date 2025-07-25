use crate::define_indicator;
use crate::indicator::{PriceSource, MAType};
use serde_json::Error;


// Acceleration Bands #加速带
define_indicator!(ACCBANDS,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (upper: f64), (middle: f64), (lower: f64)],
);

// Accumulation/Distribution Line #累积/派发线
define_indicator!(AD,
    params => [],
    output => [(timestamp: i64), (ad: f64)],
);

// Average Directional Movement Index #平均方向性指数
define_indicator!(ADOSC,
    params => [(fast_period: i32), (slow_period: i32)],
    output => [(timestamp: i64), (adosc: f64)],
);


// Average Directional Movement Index #平均方向性指数
define_indicator!(ADX,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (adx: f64)],
);

// Average Directional Movement Index Rating #平均方向性指数评级
define_indicator!(ADXR,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (adxr: f64)],
);


// Absolute Price Oscillator #绝对价格振荡器
define_indicator!(APO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (apo: f64)],
);



// Aroon #阿隆指标
define_indicator!(AROON,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (aroon_down: f64), (aroon_up: f64)],
);

// Aroon Oscillator #阿隆振荡器
define_indicator!(AROONOSC,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (aroon_osc: f64)],
);


// Average True Range #平均真实波幅
define_indicator!(ATR,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (atr: f64)],
);

//Bollinger Bands #布林带
define_indicator!(BBands,
    params => [(time_period: i32), (dev_up: f64), (dev_down: f64), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (upper: f64), (middle: f64), (lower: f64)],
);


// Balance of Power #平衡力量
define_indicator!(BOP,
    params => [],
    output => [(timestamp: i64), (bop: f64)],
);


//Commodity Channel Index #商品通道指数
define_indicator!(CCI,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (cci: f64)],
);



// 蜡烛图形态识别函数

define_indicator!(CDL2CROWS,
    params => [],
    output => [(timestamp: i64), (two_crows: i32)],
);

// Three Black Crows - 三只黑乌鸦
define_indicator!(CDL3BLACKCROWS,
    params => [],
    output => [(timestamp: i64), (three_black_crows: i32)],
);

//CDL3INSIDE 三内部上升/下降
define_indicator!(CDL3INSIDE,
    params => [],
    output => [(timestamp: i64), (three_inside: i32)],
);

//CDL3LINESTRIKE 三线打击
define_indicator!(CDL3LINESTRIKE,
    params => [],
    output => [(timestamp: i64), (three_line_strike: i32)],
);

//CDL3OUTSIDE 三外部上升/下降
define_indicator!(CDL3OUTSIDE,
    params => [],
    output => [(timestamp: i64), (three_outside: i32)],
);
//CDL3STARSINSOUTH 南方三星
define_indicator!(CDL3STARSINSOUTH,
    params => [],
    output => [(timestamp: i64), (three_stars_in_south: i32)],
);

//CDL3WHITESOLDIERS 三白兵
define_indicator!(CDL3WHITESOLDIERS,
    params => [],
    output => [(timestamp: i64), (three_white_soldiers: i32)],
);

//CDLABANDONEDBABY 弃婴
define_indicator!(CDLABANDONEDBABY,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (cdl_abandoned_baby: i32)],
);

//CDLADVANCEBLOCK 前进白线
define_indicator!(CDLADVANCEBLOCK,
    params => [],
    output => [(timestamp: i64), (cdl_advance_block: i32)],
);

//CDLBELTHOLD 支撑突破
define_indicator!(CDLBELTHOLD,
    params => [],
    output => [(timestamp: i64), (belthold: i32)],
);

//CDLBREAKAWAY 突破
define_indicator!(CDLBREAKAWAY,
    params => [],
    output => [(timestamp: i64), (breakaway: i32)],
);

//CDLCLOSINGMARUBOZU 收盘缺影线
define_indicator!(CDLCLOSINGMARUBOZU,
    params => [],
    output => [(timestamp: i64), (closing_marubozu: i32)],
);

//CDLCONCEALBABYSWALL 隐藏的婴儿
define_indicator!(CDLCONCEALBABYSWALL,
    params => [],
    output => [(timestamp: i64), (conceal_baby_wall: i32)],
);

//CDLCOUNTERATTACK 反击线
define_indicator!(CDLCOUNTERATTACK,
    params => [],
    output => [(timestamp: i64), (counter_attack: i32)],
);

//CDLDARKCLOUDCOVER 乌云盖顶
define_indicator!(CDLDARKCLOUDCOVER,
    params => [(penetration: f64)],
    output => [(timestamp: i64), (dark_cloud_cover: i32)],
);

//CDLDOJI 十字星
define_indicator!(CDLDOJI,
    params => [],
    output => [(timestamp: i64), (doji: i32)],
);


// Moving Average Convergence Divergence #移动平均收敛发散
define_indicator!(MACD,
    params => [(fast_period: i32), (slow_period: i32), (signal_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);

// Moving Average #移动平均线
define_indicator!(MA,
    params => [(time_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (ma: f64)],
);


// Relative Strength Index #相对强弱指数
define_indicator!(RSI,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rsi: f64)],
);







//Chande Momentum Oscillator #钱德动量摆动指标
define_indicator!(CMO,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (cmo: f64)],
);

//Directional Movement Index #方向性运动指数
define_indicator!(DX,
    params => [(time_period: i32)],
    output => [(timestamp: i64), (dx: f64)],
);

// MACD with controllable MA type # 可控MA类型的MACD
define_indicator!(MACDEXT,
    params => [
        (fast_period: i32),
        (fast_ma_type: MAType),
        (slow_period: i32), 
        (slow_ma_type: MAType),
        (signal_period: i32), 
        (signal_ma_type: MAType),
        (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);

//Moving Average Convergence/Divergence Fix 12/26 #移动平均收敛/发散修正12/26
define_indicator!(MACDFIX,
    params => [(signal_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (macd: f64), (signal: f64), (histogram: f64)],
);

//Money Flow Index #资金流量指数
define_indicator!(MFI,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (mfi: f64)],
);

//Minus Directional Indicator #负方向性指标
define_indicator!(MinusDi,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (minus_di: f64)],
);

// Minus Directional Movement #负方向性运动
define_indicator!(MinusDm,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (minus_dm: f64)],
);


// Momentum #动量
define_indicator!(MOM,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (momentum: f64)],
);

// Plus Directional Indicator #正方向性指标
define_indicator!(PlusDi,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (plus_di: f64)],
);

// Plus Directional Movement #正方向性运动
define_indicator!(PlusDm,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (plus_dm: f64)],
);

// Percentage Price Oscillator #百分比价格振荡器
define_indicator!(PPO,
    params => [(fast_period: i32), (slow_period: i32), (ma_type: MAType), (price_source: PriceSource)],
    output => [(timestamp: i64), (ppo: f64)],
);

// Rate of change : ((price/prevPrice)-1)*100
define_indicator!(ROC,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (roc: f64)],
);

// Rate of change Percentage: (price-prevPrice)/prevPrice
define_indicator!(ROCP,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rocp: f64)],
);

// Rate of change ratio: (price/prevPrice)
define_indicator!(ROCR,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rocr: f64)],
);

// Rate of change ratio 100 scale: (price/prevPrice)*100
define_indicator!(ROCR100,
    params => [(time_period: i32), (price_source: PriceSource)],
    output => [(timestamp: i64), (rocr100: f64)],
);

// Stochastic #随机指标
define_indicator!(STOCH,
    params => [
        (fast_k_period: i32), 
        (slow_k_period: i32), 
        (slow_k_ma_type: MAType), 
        (slow_d_period: i32), 
        (slow_d_ma_type: MAType), 
        (price_source: PriceSource)],
    output => [(timestamp: i64), (slow_k: f64), (slow_d: f64)],
);

// Stochastic Fast #快速随机指标
define_indicator!(STOCHF,
    params => [
        (fast_k_period: i32), 
        (fast_d_period: i32), 
        (fast_d_ma_type: MAType), 
        (slow_k_period: i32), 
        (price_source: PriceSource)],
    output => [(timestamp: i64), (fast_k: f64), (fast_d: f64)],
);

// Stochastic Relative Strength Index #随机相对强弱指数
define_indicator!(STOCHRSI,
    params => [
        (time_period: i32), 
        (fast_k_period: i32), 
        (fast_d_period: i32), 
        (fast_d_ma_type: MAType), 
        (price_source: PriceSource)],
    output => [(timestamp: i64), (fast_k: f64), (fast_d: f64)],
);

// Ultimate Oscillator #终极指标
define_indicator!(ULTOSC,
    params => [
        (time_period1: i32), 
        (time_period2: i32), 
        (time_period3: i32), 
        (price_source: PriceSource)],
    output => [(timestamp: i64), (ultosc: f64)],
);








