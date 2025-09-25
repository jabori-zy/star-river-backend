use crate::indicator_engine::talib::TALib;
use crate::{calculate_fn, calculate_fn_snake};
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::indicator_define::cycle::*;
use star_river_core::market::Kline;

use crate::indicator_engine::calculate::CalculateIndicatorFunction;

impl CalculateIndicatorFunction {
    // HT_DCPERIOD - Hilbert Transform - Dominant Cycle Period #希尔伯特变换 - 主导周期
    calculate_fn_snake!(HtDcperiod,
        talib_params => []
    );

    // HT_DCPHASE - Hilbert Transform - Dominant Cycle Phase #希尔伯特变换 - 主导周期相位
    calculate_fn_snake!(HtDcphase,
        talib_params => []
    );

    // HT_PHASOR - Hilbert Transform - Phasor Components #希尔伯特变换 - 相量分量
    calculate_fn_snake!(HtPhasor, // 希尔伯特变换 - 相量分量
        talib_params => []
    );

    // HT_SINE - Hilbert Transform - SineWave #希尔伯特变换 - 正弦波
    calculate_fn_snake!(HtSine,
        talib_params => []
    );

    // HT_TRENDMODE - Hilbert Transform - Trend vs Cycle Mode #希尔伯特变换 - 趋势与周期模式
    calculate_fn_snake!(HtTrendmode,
        talib_params => []
    );
}
