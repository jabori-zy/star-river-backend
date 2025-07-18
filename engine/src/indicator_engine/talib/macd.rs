use crate::indicator_engine::talib_bindings::*;
use super::TALib;
use crate::indicator_engine::talib_error::TalibError;

impl TALib {
    //Moving Average Convergence/Divergence  指数平滑异同移动平均线
    // fast_period: 快速移动平均线周期
    // slow_period: 慢速移动平均线周期
    // signal_period: 信号线周期
    // 返回格式: Vec<Vec<f64>>，每个内部向量包含 [macd, signal, histogram]
    pub fn moving_average_convergence_divergence(data: &[f64], fast_period: i32, slow_period: i32, signal_period: i32) -> Result<Vec<Vec<f64>>, TalibError> {
        unsafe {
            let mut out_begin: i32 = 0;
            let mut out_size: i32 = 0;
            let input_size = data.len() as i32;
            
            // 使用 TA_MACD_Lookback 计算精确的 lookback 期
            let lookback = TA_MACD_Lookback(fast_period, slow_period, signal_period);
            
            // 检查输入数据是否足够（需要 lookback + 1 个数据点）
            let min_required_size = lookback + 1;
            if input_size < min_required_size {
                return Err(TalibError::CalculateMACDError { 
                    fast_period, 
                    slow_period, 
                    signal_period, 
                    error: format!("Input data size ({}) is too small, need at least {} data points", input_size, min_required_size)
                });
            }

            // 计算精确的输出大小
            let expected_out_size = input_size - lookback;
            let mut out_macd: Vec<f64> = vec![0.0; expected_out_size as usize];
            let mut out_signal: Vec<f64> = vec![0.0; expected_out_size as usize];
            let mut out_hist: Vec<f64> = vec![0.0; expected_out_size as usize];

            let ret = TA_MACD(
                0,                          // start index
                input_size - 1,             // end index
                data.as_ptr(),             // input data
                fast_period,               // fast period
                slow_period,               // slow period
                signal_period,             // signal period
                &mut out_begin,            // output begin index
                &mut out_size,             // output size
                out_macd.as_mut_ptr(),     // MACD line output
                out_signal.as_mut_ptr(),   // signal line output
                out_hist.as_mut_ptr(),     // histogram output
            );

            if ret != TA_RetCode_TA_SUCCESS {
                return Err(TalibError::CalculateMACDError { 
                    fast_period, 
                    slow_period, 
                    signal_period, 
                    error: format!("talib error code: {:?}", ret) 
                });
            }

            // 验证结果并调整向量大小
            if out_size as usize != expected_out_size as usize {
                out_macd.truncate(out_size as usize);
                out_signal.truncate(out_size as usize);
                out_hist.truncate(out_size as usize);
            }
            
            // 构建嵌套列表结果，每个内部向量包含 [macd, signal, histogram]
            let mut result = Vec::with_capacity(out_size as usize);
            for i in 0..out_size as usize {
                result.push(vec![out_macd[i], out_signal[i], out_hist[i]]);
            }
            
            Ok(result)
        }
    }
}
