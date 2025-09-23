use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::talib_bindings::*;
// use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use chrono::{DateTime, Utc};
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::indicator_define::volume::*;

impl TALib {
    // AD                   Chaikin A/D Line #钱德动量线
    talib_fn!(
        AD,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [],
        output => [(ad: f64)],
    );

    // ADOSC                Chaikin A/D Oscillator #钱德动量振荡器
    talib_fn!(
        ADOSC,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64]), (volume: &[f64])],
        talib_params => [(fast_period: i32), (slow_period: i32)],
        output => [(adosc: f64)],
    );

    // OBV                  On Balance Volume #能量潮
    talib_fn!(
        OBV,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(close: &[f64]), (volume: &[f64])],
        talib_params => [],
        output => [(obv: f64)],
    );
}
