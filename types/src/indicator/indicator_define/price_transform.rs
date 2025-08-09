use crate::{define_indicator};

// AVGPRICE             Average Price #平均价格
define_indicator!(AVGPRICE,
    params => [],
    output => [(timestamp: i64), (avgprice: f64)],
);

// MEDPRICE             Median Price #中位数价格
define_indicator!(MEDPRICE,
    params => [],
    output => [(timestamp: i64), (medprice: f64)],
);

// TYPPRICE             Typical Price #典型价格
define_indicator!(TYPPRICE,
    params => [],
    output => [(timestamp: i64), (typprice: f64)],
);

// WCLPRICE             Weighted Close Price #加权收盘价
define_indicator!(WCLPRICE,
    params => [],
    output => [(timestamp: i64), (wclprice: f64)],
);