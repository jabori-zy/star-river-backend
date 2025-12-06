use ta_lib::{Indicator, indicator::momentum::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn, calculate_fn_snake};

impl CalculateIndicatorFunction {
    // ADX - Average Directional Movement Index
    calculate_fn!(ADX,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // ADXR - Average Directional Movement Index Rating
    calculate_fn!(ADXR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // APO - Absolute Price Oscillator
    calculate_fn!(APO,
        input => [close],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (ma_type: MAType),
        ]
    );

    // AROON - Aroon
    calculate_fn!(AROON,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // AROONOSC - Aroon Oscillator
    calculate_fn!(AROONOSC,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // BOP - Balance Of Power
    calculate_fn!(BOP,
        input => [open,high,low,close]
    );

    // CCI - Commodity Channel Index
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

    // DX - Directional Movement Index
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

    // MACDEXT - MACD with controllable MA type
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

    // MACDFIX - Moving Average Convergence/Divergence Fix 12/26
    calculate_fn!(MACDFIX,
        input => [close],
        talib_params => [
            (signal_period: i32),
        ]
    );

    // MFI - Money Flow Index
    calculate_fn!(MFI,
        input => [high,low,close,volume],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MINUS_DI - Minus Directional Indicator
    calculate_fn_snake!(MinusDi,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MINUS_DM - Minus Directional Movement
    calculate_fn_snake!(MinusDm,
        input => [high,low],
        talib_params => [
            (time_period: i32),
        ]
    );

    // MOM - Momentum
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

    // ROC - Rate of change : ((price/prevPrice)-1)*100
    calculate_fn!(ROC,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ROCP - Rate of change Percentage: (price-prevPrice)/prevPrice
    calculate_fn!(ROCP,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ROCR - Rate of change ratio: (price/prevPrice)
    calculate_fn!(ROCR,
        talib_params => [
            (time_period: i32),
        ]
    );

    // ROCR100 - Rate of change ratio 100 scale: (price/prevPrice)*100
    calculate_fn!(ROCR100,
        talib_params => [
            (time_period: i32),
        ]
    );

    // RSI - Relative Strength Index
    calculate_fn!(RSI,
        talib_params => [
            (time_period: i32),
        ]
    );

    // STOCH - Stochastic
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

    // STOCHF - Stochastic Fast
    calculate_fn!(STOCHF,
        input => [high,low,close],
        talib_params => [
            (fast_k_period: i32),
            (fast_d_period: i32),
            (fast_d_ma_type: MAType),
        ]
    );

    // STOCHRSI - Stochastic Relative Strength Index
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

    // ULTOSC - Ultimate Oscillator
    calculate_fn!(ULTOSC,
        input => [high,low,close],
        talib_params => [
            (time_period1: i32),
            (time_period2: i32),
            (time_period3: i32),
        ]
    );

    // WILLR - Williams' %R
    calculate_fn!(WILLR,
        input => [high, low, close],
        talib_params => [
            (time_period: i32),
        ]
    );
}
