pub mod lookback;
pub mod talib_fn_define;
pub mod talib_macros;
    use crate::indicator_engine::talib_bindings::*;



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
