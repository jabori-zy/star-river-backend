use crate::define_indicator;
use chrono::{DateTime, Utc};

// AVGPRICE             Average Price #平均价格
define_indicator!(AVGPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (avgprice: f64)],
);

// MEDPRICE             Median Price #中位数价格
define_indicator!(MEDPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (medprice: f64)],
);

// TYPPRICE             Typical Price #典型价格
define_indicator!(TYPPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (typprice: f64)],
);

// WCLPRICE             Weighted Close Price #加权收盘价
define_indicator!(WCLPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (wclprice: f64)],
);
