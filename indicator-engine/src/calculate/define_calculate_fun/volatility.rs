use ta_lib::{Indicator, indicator::volatility::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn};

impl CalculateIndicatorFunction {
    // ATR - Average True Range
    calculate_fn!(ATR,
        input => [high,low,close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // NATR - Normalized Average True Range
    calculate_fn!(NATR,
        input => [high, low, close],
        talib_params => [
            (time_period: i32),
        ]
    );

    // TRANGE - True Range
    calculate_fn!(TRANGE,
        input => [high, low, close]
    );
}
