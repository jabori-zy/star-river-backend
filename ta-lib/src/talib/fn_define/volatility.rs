use crate::talib::TALib;
use crate::talib_bindings::*;
// use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use chrono::{DateTime, Utc};
use crate::Indicator;
use crate::indicator::volatility::*;

impl TALib {
    // ATR                  Average True Range #平均真实波幅 #平均真实波幅
    talib_fn!(
        ATR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(atr: f64)],
    );

    // NATR                 Normalized Average True Range #归一化平均真实波幅
    talib_fn!(
        NATR,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [(time_period: i32)],
        output => [(natr: f64)],
    );

    // TRANGE               True Range #真实波幅
    talib_fn!(
        TRANGE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(trange: f64)],
    );
}
