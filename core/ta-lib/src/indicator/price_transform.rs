use chrono::{DateTime, Utc};

use crate::define_indicator;

// AVGPRICE             Average Price
define_indicator!(AVGPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (avgprice: Option<f64>)],
);

// MEDPRICE             Median Price
define_indicator!(MEDPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (medprice: Option<f64>)],
);

// TYPPRICE             Typical Price
define_indicator!(TYPPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (typprice: Option<f64>)],
);

// WCLPRICE             Weighted Close Price
define_indicator!(WCLPRICE,
    params => [],
    output => [(datetime: DateTime<Utc>), (wclprice: Option<f64>)],
);
