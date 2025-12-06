use ta_lib::{Indicator, indicator::volume::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn};

impl CalculateIndicatorFunction {
    // AD - Chaikin A/D Line
    calculate_fn!(AD,
        input => [high,low,close,volume]
    );

    // ADOSC - Chaikin A/D Oscillator
    calculate_fn!(ADOSC,
        input => [high,low,close,volume],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
        ]
    );

    // OBV - On Balance Volume
    calculate_fn!(OBV,
        input => [close, volume]
    );
}
