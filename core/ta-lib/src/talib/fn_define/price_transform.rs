use chrono::{DateTime, Utc};

// use crate::indicator_engine::talib_error::TalibError;
use crate::{Indicator, indicator::price_transform::*, talib_fn};
use crate::{talib::TALib, talib_bindings::*};

impl TALib {
    // AVGPRICE             Average Price
    talib_fn!(
        AVGPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(avgprice: f64)],
    );

    // MEDPRICE             Median Price
    talib_fn!(
        MEDPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [],
        output => [(medprice: f64)],
    );

    // TYPPRICE             Typical Price
    talib_fn!(
        TYPPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(typprice: f64)],
    );

    // WCLPRICE             Weighted Close Price
    talib_fn!(
        WCLPRICE,
        datetime => (datetime_list: &[DateTime<Utc>]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(wclprice: f64)],
    );
}
