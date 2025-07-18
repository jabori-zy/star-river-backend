use crate::indicator_engine::talib_bindings::*;
use super::TALib;
use crate::indicator_engine::talib_error::TalibError;

impl TALib {
    //Bollinger Bands  布林带
    // period: 周期
    // dev_up: 上轨标准差倍数
    // dev_down: 下轨标准差倍数
    // ma_type: 移动平均类型
    pub fn bollinger_bands(data: &[f64], period: i32, dev_up: f64, dev_down: f64, ma_type: i32) -> Result<Vec<Vec<f64>>, TalibError> {
        unsafe {
            let mut out_begin: i32 = 0;
            let mut out_size: i32 = 0;
            let input_size = data.len() as i32;

            let lookback = TA_BBANDS_Lookback(period, dev_up, dev_down, ma_type);
            let expected_out_size = input_size - lookback;

            let mut out_upper: Vec<f64> = vec![0.0; expected_out_size as usize];
            let mut out_middle: Vec<f64> = vec![0.0; expected_out_size as usize];
            let mut out_lower: Vec<f64> = vec![0.0; expected_out_size as usize];

            let ret = TA_BBANDS(
                0,
                input_size - 1,
                data.as_ptr(),
                period,
                dev_up,
                dev_down,
                ma_type,
                &mut out_begin,
                &mut out_size,
                out_upper.as_mut_ptr(),
                out_middle.as_mut_ptr(),
                out_lower.as_mut_ptr(),
            );

            if ret != TA_RetCode_TA_SUCCESS {
                return Err(TalibError::CalculateBBANDSError { 
                    period, 
                    dev_up, 
                    dev_down, 
                    ma_type, 
                    error: format!("talib error code: {:?}", ret) 
                });
            }

            if out_size as usize != expected_out_size as usize {
                out_upper.truncate(out_size as usize);
                out_middle.truncate(out_size as usize);
                out_lower.truncate(out_size as usize);
            }

            let mut result = Vec::with_capacity(out_size as usize);
            for i in 0..out_size as usize {
                result.push(vec![out_upper[i], out_middle[i], out_lower[i]]);
            }

            Ok(result)
        }
    }
}
