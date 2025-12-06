use ta_lib::{Indicator, indicator::cycle::*};

use crate::{calculate::CalculateIndicatorFunction, calculate_fn_snake};

impl CalculateIndicatorFunction {
    // HT_DCPERIOD - Hilbert Transform - Dominant Cycle Period
    calculate_fn_snake!(HtDcperiod,
        talib_params => []
    );

    // HT_DCPHASE - Hilbert Transform - Dominant Cycle Phase
    calculate_fn_snake!(HtDcphase,
        talib_params => []
    );

    // HT_PHASOR - Hilbert Transform - Phasor Components
    calculate_fn_snake!(HtPhasor,
        talib_params => []
    );

    // HT_SINE - Hilbert Transform - SineWave
    calculate_fn_snake!(HtSine,
        talib_params => []
    );

    // HT_TRENDMODE - Hilbert Transform - Trend vs Cycle Mode
    calculate_fn_snake!(HtTrendmode,
        talib_params => []
    );
}
