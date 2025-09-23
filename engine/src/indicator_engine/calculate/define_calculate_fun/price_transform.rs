use crate::calculate_fn;
use crate::indicator_engine::calculate::CalculateIndicatorFunction;
use crate::indicator_engine::talib::TALib;
use star_river_core::cache::CacheValue;
use star_river_core::error::engine_error::indicator_engine_error::*;
use star_river_core::indicator::Indicator;
use star_river_core::indicator::indicator_define::price_transform::*;
use std::sync::Arc;

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
