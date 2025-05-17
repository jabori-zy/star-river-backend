use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;
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

    pub fn sma(data: &[f64], period: i32) -> Result<Vec<f64>, TalibError> {
        unsafe {
            // 查找第一个非NaN值的位置
            let mut begin_idx = 0;
            for (i, &value) in data.iter().enumerate() {
                if value.is_nan() {
                    begin_idx = i + 1;
                } else {
                    break;
                }
            }
            
            // 检查有效数据是否足够
            if begin_idx >= data.len() {
                return Ok(Vec::new()); // 全部是NaN值
            }
            
            // 计算lookback期
            let lookback = TA_SMA_Lookback(period);
            let valid_len = data.len() - begin_idx;
            
            // 检查剩余数据长度是否足够
            if valid_len <= lookback as usize {
                return Ok(Vec::new()); // 有效数据不足以计算SMA
            }
            
            // 分配结果空间
            let result_size = valid_len - lookback as usize;
            let mut out: Vec<f64> = vec![0.0; result_size];
            let mut out_begin: i32 = 0;
            let mut out_size: i32 = 0;

            let ret = TA_MA(
                0,                                // startIdx (相对于有效数据的起始)
                (valid_len - 1) as i32,           // endIdx (相对于有效数据的结束)
                data[begin_idx..].as_ptr(),       // 从第一个有效数据开始
                period,                           // optInTimePeriod
                TA_MAType_TA_MAType_SMA as i32,
                &mut out_begin,
                &mut out_size,
                out.as_mut_ptr(),
            );

            if ret != TA_RetCode_TA_SUCCESS {
                return Err(TalibError::CalculateSMAError { period, error: format!("talib error code: {:?}", ret) });
            }

            // 验证结果并返回
            if out_size as usize != result_size {
                out.truncate(out_size as usize);
            }
            
            Ok(out)
        }
    }

    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
