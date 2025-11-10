use chrono::{DateTime, Utc};

// use crate::indicator_engine::talib_error::TalibError;
use crate::{Indicator, indicator::cycle::*, talib_snake_fn};
use crate::{talib::TALib, talib_bindings::*};

impl TALib {
    // HT_DCPERIOD          Hilbert Transform - Dominant Cycle Period #希尔伯特变换 - 主导周期
    talib_snake_fn!(
        HtDcperiod,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [],
        output => [(ht_dcperiod: f64)],
    );

    // HT_DCPHASE           Hilbert Transform - Dominant Cycle Phase #希尔伯特变换 - 主导周期相位
    talib_snake_fn!(
        HtDcphase,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [],
        output => [(ht_dcphase: f64)],
    );

    // HT_PHASOR            Hilbert Transform - Phasor Components #希尔伯特变换 - 相量分量
    talib_snake_fn!(
        HtPhasor,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [],
        output => [(in_phase: f64), (quadrature: f64)],
    );

    // HT_SINE              Hilbert Transform - SineWave #希尔伯特变换 - 正弦波
    talib_snake_fn!(
        HtSine,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [],
        output => [(sine: f64), (lead_sine: f64)],
    );

    // HT_TRENDMODE         Hilbert Transform - Trend vs Cycle Mode #希尔伯特变换 - 趋势与周期模式
    talib_snake_fn!(
        HtTrendmode,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(data: &[f64])],
        talib_params => [],
        output => [(ht_trendmode: i32)],
    );
}
