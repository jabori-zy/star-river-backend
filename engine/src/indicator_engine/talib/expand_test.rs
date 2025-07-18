
use crate::define_indicator;
use crate::indicator_engine::talib::indicators::sma_calculate;
use crate::indicator_engine::talib::indicator_meta::*;
use crate::indicator_engine::talib_bindings::*;

define_indicator!(SMA, {
    group: Overlap,
    input: Single,
    output: Single,
    description: "Simple Moving Average",
    lookback: |params| unsafe { TA_SMA_Lookback(params[0].as_period()) },
    calculate: sma_calculate,
});


// define_indicator!(DUAL_MA_CROSS, {
//     group: Momentum,
//     input: Single,
//     output: Single,
//     description: "Dual Moving Average Crossover Signal",
//     lookback: |params| {
//         let fast_period = params[0].as_fast_period();
//         let slow_period = params[1].as_slow_period();
//         unsafe { TA_SMA_Lookback(slow_period.max(fast_period)) }
//     },
//     calculate: dual_ma_cross_calculate,
// });