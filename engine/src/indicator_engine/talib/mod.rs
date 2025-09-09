// pub mod rsi;
// pub mod ma;
// pub mod macd;
// pub mod bbands;
pub mod lookback;
pub mod talib_fn_define;
pub mod talib_macros;

use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;
use crate::talib_fn;
use crate::talib_snake_fn;
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

    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
