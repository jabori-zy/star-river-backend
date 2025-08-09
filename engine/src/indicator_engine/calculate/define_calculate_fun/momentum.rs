
use crate::calculate_fn;
use crate::calculate_fn_snake;
use types::indicator::indicator_define::momentum::*;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use types::cache::CacheValue;
use types::indicator::Indicator;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;

impl CalculateIndicatorFunction {
    // ADX - Average Directional Movement Index #平均方向性指数
    calculate_fn!(ADX,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // ADXR - Average Directional Movement Index Rating #平均方向性指数评级
    calculate_fn!(ADXR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // APO - Absolute Price Oscillator #绝对价格振荡器
    calculate_fn!(APO,
        input => [close],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (ma_type: MAType),
        ]
    );

    // AROON - Aroon #阿隆指标
    calculate_fn!(AROON,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // AROONOSC - Aroon Oscillator #阿隆振荡器
    calculate_fn!(AROONOSC,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // BOP - Balance Of Power #平衡力量
    calculate_fn!(BOP,
        input => [open,high,low,close]
    );

    // CCI - Commodity Channel Index #商品通道指数
    calculate_fn!(CCI,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // CMO - Chande Momentum Oscillator
    calculate_fn!(CMO,
        input => [close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // DX - Directional Movement Index #方向性指数
    calculate_fn!(DX,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MACD - Moving Average Convergence/Divergence
    calculate_fn!(MACD,
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (signal_period: i32),
        ]
    );

    // MACDEXT - MACD with controllable MA type #可控MA类型的MACD
    calculate_fn!(MACDEXT,
        input => [close],
        talib_params => [
            (fast_period: i32),
            (fast_ma_type: MAType),
            (slow_period: i32),
            (slow_ma_type: MAType),
            (signal_period: i32),
            (signal_ma_type: MAType),
        ]
    );

    // MACDFIX - Moving Average Convergence/Divergence Fix 12/26 #12/26固定MACD
    calculate_fn!(MACDFIX,
        input => [close],
        talib_params => [
            (signal_period: i32),
        ]
    );

    // MFI - Money Flow Index #资金流量指数
    calculate_fn!(MFI,
        input => [high,low,close,volume],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MINUS_DI - Minus Directional Indicator #负方向性指标
    calculate_fn_snake!(MinusDi,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MINUS_DM - Minus Directional Movement #负方向性运动
    calculate_fn_snake!(MinusDm,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MOM - Momentum #动量
    calculate_fn!(MOM,
        input => [close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // PLUS_DI - Plus Directional Indicator
    calculate_fn_snake!(PlusDi,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // PLUS_DM - Plus Directional Movement
    calculate_fn_snake!(PlusDm,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // PPO - Percentage Price Oscillator
    calculate_fn!(PPO,
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (ma_type: MAType),
        ]
    );

    // ROC - Rate of change : ((price/prevPrice)-1)*100 #变化率
    calculate_fn!(ROC,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ROCP - Rate of change Percentage: (price-prevPrice)/prevPrice #变化率百分比
    calculate_fn!(ROCP,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ROCR - Rate of change ratio: (price/prevPrice) #变化率比率
    calculate_fn!(ROCR,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ROCR100 - Rate of change ratio 100 scale: (price/prevPrice)*100 #变化率比率100比例
    calculate_fn!(ROCR100,
        talib_params => [
            (time_period: i32),
        ]
    );

    // RSI - Relative Strength Index #相对强弱指数
    calculate_fn!(RSI,
        talib_params => [
            (time_period: i32),
        ]
    );

    // STOCH - Stochastic #随机指标
    calculate_fn!(STOCH,
        input => [high,low,close],
        talib_params => [
            (fast_k_period: i32), 
            (slow_k_period: i32), 
            (slow_k_ma_type: MAType), 
            (slow_d_period: i32), 
            (slow_d_ma_type: MAType),
        ]
    );

    // STOCHF - Stochastic Fast #快速随机指标
    calculate_fn!(STOCHF,
        input => [high,low,close],
        talib_params => [
            (fast_k_period: i32), 
            (fast_d_period: i32), 
            (fast_d_ma_type: MAType),
        ]
    );

    // STOCHRSI - Stochastic Relative Strength Index #随机相对强弱指数
    calculate_fn!(STOCHRSI,
        input => [close],
        talib_params => [
            (time_period: i32), 
            (fast_k_period: i32), 
            (fast_d_period: i32), 
            (fast_d_ma_type: MAType),
        ]
    );

    // TRIX - 1-day Rate-Of-Change (ROC) of a Triple Smooth EMA
    calculate_fn!(TRIX,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ULTOSC - Ultimate Oscillator #终极振荡器
    calculate_fn!(ULTOSC,
        input => [high,low,close],
        talib_params => [
            (time_period1: i32), 
            (time_period2: i32),
            (time_period3: i32),
        ]
    );

    // WILLR - Williams' %R #威廉指标
    calculate_fn!(WILLR,
        input => [high, low, close],
        talib_params => [
            (time_period: i32),
        ]
    );
}