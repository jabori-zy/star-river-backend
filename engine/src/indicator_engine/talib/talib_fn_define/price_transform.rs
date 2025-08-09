use crate::talib_fn;
use types::indicator::indicator_define::price_transform::*;
use crate::indicator_engine::talib::TALib;
use types::indicator::Indicator;
use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;

impl TALib {

    // AVGPRICE             Average Price #平均价格
    talib_fn!(
        AVGPRICE,
        timestamp => (timestamp_list: &[i64]),
        input => [(open: &[f64]), (high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(avgprice: f64)],
    );

    // MEDPRICE             Median Price #中位数价格
    talib_fn!(
        MEDPRICE,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64])],
        talib_params => [],
        output => [(medprice: f64)],
    );

    // TYPPRICE             Typical Price #典型价格
    talib_fn!(
        TYPPRICE,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(typprice: f64)],
    );

    // WCLPRICE             Weighted Close Price #加权收盘价
    talib_fn!(
        WCLPRICE,
        timestamp => (timestamp_list: &[i64]),
        input => [(high: &[f64]), (low: &[f64]), (close: &[f64])],
        talib_params => [],
        output => [(wclprice: f64)],
    );
}
