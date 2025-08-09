use crate::calculate_fn;
use types::indicator::indicator_define::price_transform::*;
use types::cache::CacheValue;
use types::indicator::Indicator;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;

impl CalculateIndicatorFunction {
    // AVGPRICE - Average Price #平均价格
    calculate_fn!(AVGPRICE,
        input => [open, high, low, close]
    );

    // MEDPRICE - Median Price #中位数价格
    calculate_fn!(MEDPRICE,
        input => [high, low]
    );

    // TYPPRICE - Typical Price #典型价格
    calculate_fn!(TYPPRICE,
        input => [high, low, close]
    );

    // WCLPRICE - Weighted Close Price #加权收盘价
    calculate_fn!(WCLPRICE,
        input => [high, low, close]
    );
}