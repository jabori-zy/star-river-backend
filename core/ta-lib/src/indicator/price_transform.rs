use chrono::{DateTime, Utc};

use crate::define_indicator;

// AVGPRICE             Average Price #平均价格
define_indicator!(AVGPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (avgprice: Option<f64>)],
);

// MEDPRICE             Median Price #中位数价格
define_indicator!(MEDPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (medprice: Option<f64>)],
);

// TYPPRICE             Typical Price #典型价格
define_indicator!(TYPPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (typprice: Option<f64>)],
);

// WCLPRICE             Weighted Close Price #加权收盘价
define_indicator!(WCLPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (wclprice: Option<f64>)],
);
