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

    pub fn sma(data: &[f64], period: i32) -> Result<Vec<f64>, String> {
        unsafe {
            let mut out: Vec<f64> = vec![0.0; data.len()];
            let mut out_begin: i32 = 0;
            let mut out_size: i32 = 0;

            let ret = TA_MA(
                0,                       // startIdx
                (data.len() - 1) as i32, // endIdx
                data.as_ptr(),           // inReal
                period,                  // optInTimePeriod
                TA_MAType_TA_MAType_SMA as i32,
                &mut out_begin,
                &mut out_size,
                out.as_mut_ptr(),
            );

            if ret != TA_RetCode_TA_SUCCESS {
                return Err("Failed to calculate SMA".to_string());
            }

            // out.drain(0..out_begin as usize);
            // out.truncate(out_size as usize);
            out.reverse();
            Ok(out)
        }
    }

    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
