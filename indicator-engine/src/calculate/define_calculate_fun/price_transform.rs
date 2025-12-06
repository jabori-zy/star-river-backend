use ta_lib::Indicator;

use crate::{calculate::CalculateIndicatorFunction, calculate_fn};

impl CalculateIndicatorFunction {
    // AVGPRICE - Average Price
    calculate_fn!(AVGPRICE,
        input => [open, high, low, close]
    );

    // MEDPRICE - Median Price
    calculate_fn!(MEDPRICE,
        input => [high, low]
    );

    // TYPPRICE - Typical Price
    calculate_fn!(TYPPRICE,
        input => [high, low, close]
    );

    // WCLPRICE - Weighted Close Price
    calculate_fn!(WCLPRICE,
        input => [high, low, close]
    );
}
