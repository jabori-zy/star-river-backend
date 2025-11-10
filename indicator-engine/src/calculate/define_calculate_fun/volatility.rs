use ta_lib::{Indicator, indicator::volatility::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn};

impl CalculateIndicatorFunction {
    // ATR - Average True Range #平均真实波幅
    calculate_fn!(ATR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // NATR - Normalized Average True Range #归一化平均真实波幅
    calculate_fn!(NATR,
        input => [high, low, close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // TRANGE - True Range #真实波幅
    calculate_fn!(TRANGE,
        input => [high, low, close]
    );
}
