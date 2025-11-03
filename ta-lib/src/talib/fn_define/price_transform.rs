use crate::talib::TALib;
use crate::talib_bindings::*;
// use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use chrono::{DateTime, Utc};
use crate::Indicator;
use crate::indicator::price_transform::*;

impl TALib {
    // AVGPRICE             Average Price #平均价格
    talib_fn!(
        AVGPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(avgprice: f64)],
    );

    // MEDPRICE             Median Price #中位数价格
    talib_fn!(
        MEDPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [],
        output => [(medprice: f64)],
    );

    // TYPPRICE             Typical Price #典型价格
    talib_fn!(
        TYPPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(typprice: f64)],
    );

    // WCLPRICE             Weighted Close Price #加权收盘价
    talib_fn!(
        WCLPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(wclprice: f64)],
    );
}
