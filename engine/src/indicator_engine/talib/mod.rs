pub mod rsi;
pub mod ma;
pub mod macd;
pub mod bbands;
pub mod talib_macros;

use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;
use types::indicator::IndicatorConfig;
// use indicator_meta::*;
// use indicator_definitions::*;
use crate::talib_fn;
use types::indicator::indicator::*;
use types::indicator::Indicator;

#[derive(Clone)]
pub struct TALib;


impl TALib {

    pub fn init() -> Result<Self, String> {
        unsafe {
            let ret = TA_Initialize();
            if ret != TA_RetCode_TA_SUCCESS {
                return Err(format!("TA-Lib 初始化失败: {:?}", ret));
            }
        }

        Ok(Self)
    }

    talib_fn!(
        MA,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32),
            (ma_type: i32),
        ],
        output => [ma],
    );

    talib_fn!(
        MACD,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (signal_period: i32),
        ],
        output => [macd, signal, histogram],
    );

    talib_fn!(
        BBands,
        timestamp => (timestamp_list: &[i64]),
        input => [(data: &[f64])],
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: i32),
        ],
        output => [upper, middle, lower],
    );


    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
