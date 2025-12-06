pub mod fn_define;
pub mod lookback;
pub mod talib_macros;
use crate::talib_bindings::*;

#[derive(Clone)]
pub struct TALib;

impl TALib {
    pub fn init() -> Result<Self, String> {
        unsafe {
            let ret = TA_Initialize();
            if ret != TA_RetCode_TA_SUCCESS {
                return Err(format!("TA-Lib initialization failed: {:?}", ret));
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
