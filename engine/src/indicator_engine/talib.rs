use crate::indicator_engine::talib_bindings::*;
use crate::indicator_engine::talib_error::TalibError;
use types::indicator::IndicatorConfig;

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

    pub fn lookback(indicator_config: &IndicatorConfig) -> u32 {
        match indicator_config {
            IndicatorConfig::SMA(sma_config) => unsafe { TA_SMA_Lookback(sma_config.period) as u32 },
            IndicatorConfig::MACD(macd_config) => unsafe { TA_MACD_Lookback(macd_config.fast_period, macd_config.slow_period, macd_config.signal_period) as u32 },
            IndicatorConfig::BBands(bbands_config) => unsafe { TA_BBANDS_Lookback(bbands_config.period, bbands_config.dev_up.into_inner(), bbands_config.dev_down.into_inner(), bbands_config.ma_type.clone() as i32) as u32 },
        }
    }

    //Simple Moving Average
    pub fn simple_moving_average(data: &[f64], period: i32) -> Result<Vec<f64>, TalibError> {
        unsafe {
            
            // 计算lookback期
            let lookback = TA_SMA_Lookback(period);
            let valid_len = data.len();
            
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
                data.as_ptr(),       // 从第一个有效数据开始
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

    pub fn shutdown() {
        unsafe {
            TA_Shutdown();
        }
    }
}
