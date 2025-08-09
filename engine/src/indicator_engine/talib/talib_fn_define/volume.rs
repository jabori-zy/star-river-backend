use crate::talib_fn;
use types::indicator::indicator_define::volume::*;
use crate::indicator_engine::talib::TALib;
use types::indicator::Indicator;
use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;

impl TALib {

    // AD                   Chaikin A/D Line #钱德动量线 
    talib_fn!(
        AD,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [],
        output => [(ad: f64)],
    );

    // ADOSC                Chaikin A/D Oscillator #钱德动量振荡器
    talib_fn!(
        ADOSC,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32)],
        output => [(adosc: f64)],
    );

    // OBV                  On Balance Volume #能量潮
    talib_fn!(
        OBV,
        timestamp => (timestamp_list: &[i64]),
        input => [(close: &[f64]), (volume: &[f64])],
        talib_params => [],
        output => [(obv: f64)],
    );
}
