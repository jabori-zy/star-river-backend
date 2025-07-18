use crate::indicator_engine::talib::indicator_meta::*;
use crate::indicator_engine::talib_error::TalibError;
use crate::indicator_engine::talib_bindings::*;
use crate::define_indicator;

// 手动实现指标计算函数，避免复杂的宏

// SMA 计算函数
pub fn sma_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    if params.is_empty() {
        return Err(TalibError::GenericCalculationError {
            error: "SMA requires period parameter".to_string()
        });
    }

    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "SMA requires single input data".to_string()
        }),
    };

    let period = params[0].as_period();

    unsafe {
        let input_size = data.len() as i32;
        let lookback = TA_SMA_Lookback(period);

        if input_size <= lookback {
            return Ok(IndicatorOutput::Single(Vec::new()));
        }

        let expected_out_size = input_size - lookback;
        let mut out: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_begin: i32 = 0;
        let mut out_size: i32 = 0;

        let ret = TA_SMA(
            0,
            input_size - 1,
            data.as_ptr(),
            period,
            &mut out_begin,
            &mut out_size,
            out.as_mut_ptr(),
        );

        if ret != TA_RetCode_TA_SUCCESS {
            return Err(TalibError::GenericCalculationError {
                error: format!("TA-Lib SMA error code: {:?}", ret)
            });
        }

        if out_size as usize != expected_out_size as usize {
            out.truncate(out_size as usize);
        }

        Ok(IndicatorOutput::Single(out))
    }
}

// MACD 计算函数
pub fn macd_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    if params.len() < 3 {
        return Err(TalibError::GenericCalculationError {
            error: "MACD requires fast_period, slow_period, signal_period parameters".to_string()
        });
    }

    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "MACD requires single input data".to_string()
        }),
    };

    let fast_period = params[0].as_fast_period();
    let slow_period = params[1].as_slow_period();
    let signal_period = params[2].as_signal_period();

    unsafe {
        let input_size = data.len() as i32;
        let lookback = TA_MACD_Lookback(fast_period, slow_period, signal_period);

        if input_size <= lookback {
            return Ok(IndicatorOutput::Triple(Vec::new(), Vec::new(), Vec::new()));
        }

        let expected_out_size = input_size - lookback;
        let mut out_macd: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_signal: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_hist: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_begin: i32 = 0;
        let mut out_size: i32 = 0;

        let ret = TA_MACD(
            0,
            input_size - 1,
            data.as_ptr(),
            fast_period,
            slow_period,
            signal_period,
            &mut out_begin,
            &mut out_size,
            out_macd.as_mut_ptr(),
            out_signal.as_mut_ptr(),
            out_hist.as_mut_ptr(),
        );

        if ret != TA_RetCode_TA_SUCCESS {
            return Err(TalibError::GenericCalculationError {
                error: format!("TA-Lib MACD error code: {:?}", ret)
            });
        }

        if out_size as usize != expected_out_size as usize {
            out_macd.truncate(out_size as usize);
            out_signal.truncate(out_size as usize);
            out_hist.truncate(out_size as usize);
        }

        Ok(IndicatorOutput::Triple(out_macd, out_signal, out_hist))
    }
}

// BBands 计算函数
pub fn bbands_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    if params.len() < 4 {
        return Err(TalibError::GenericCalculationError {
            error: "BBands requires period, dev_up, dev_down, ma_type parameters".to_string()
        });
    }

    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "BBANDS requires single input data".to_string()
        }),
    };

    let period = params[0].as_period();
    let dev_up = params[1].as_dev_up();
    let dev_down = params[2].as_dev_down();
    let ma_type = params[3].as_ma_type();

    unsafe {
        let input_size = data.len() as i32;
        let lookback = TA_BBANDS_Lookback(period, dev_up, dev_down, ma_type);

        if input_size <= lookback {
            return Ok(IndicatorOutput::Triple(Vec::new(), Vec::new(), Vec::new()));
        }

        let expected_out_size = input_size - lookback;
        let mut out_upper: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_middle: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_lower: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_begin: i32 = 0;
        let mut out_size: i32 = 0;

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
            return Err(TalibError::GenericCalculationError {
                error: format!("TA-Lib BBands error code: {:?}", ret)
            });
        }

        if out_size as usize != expected_out_size as usize {
            out_upper.truncate(out_size as usize);
            out_middle.truncate(out_size as usize);
            out_lower.truncate(out_size as usize);
        }

        Ok(IndicatorOutput::Triple(out_upper, out_middle, out_lower))
    }
}

// RSI 计算函数
pub fn rsi_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    if params.is_empty() {
        return Err(TalibError::GenericCalculationError {
            error: "RSI requires period parameter".to_string()
        });
    }

    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "RSI requires single input data".to_string()
        }),
    };

    let period = params[0].as_period();

    unsafe {
        let input_size = data.len() as i32;
        let lookback = TA_RSI_Lookback(period);

        if input_size <= lookback {
            return Ok(IndicatorOutput::Single(Vec::new()));
        }

        let expected_out_size = input_size - lookback;
        let mut out: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_begin: i32 = 0;
        let mut out_size: i32 = 0;

        let ret = TA_RSI(
            0,
            input_size - 1,
            data.as_ptr(),
            period,
            &mut out_begin,
            &mut out_size,
            out.as_mut_ptr(),
        );

        if ret != TA_RetCode_TA_SUCCESS {
            return Err(TalibError::GenericCalculationError {
                error: format!("TA-Lib RSI error code: {:?}", ret)
            });
        }

        if out_size as usize != expected_out_size as usize {
            out.truncate(out_size as usize);
        }

        Ok(IndicatorOutput::Single(out))
    }
}

// 使用define_indicator宏定义指标
define_indicator!(SMA, {
    group: Overlap,
    input: Single,
    output: Single,
    description: "Simple Moving Average",
    lookback: |params| unsafe { TA_SMA_Lookback(params[0].as_period()) },
    calculate: sma_calculate,
});

define_indicator!(MACD, {
    group: Momentum,
    input: Single,
    output: Triple,
    description: "Moving Average Convergence/Divergence",
    lookback: |params| unsafe { TA_MACD_Lookback(params[0].as_fast_period(), params[1].as_slow_period(), params[2].as_signal_period()) },
    calculate: macd_calculate,
});

define_indicator!(BBANDS, {
    group: Overlap,
    input: Single,
    output: Triple,
    description: "Bollinger Bands",
    lookback: |params| unsafe { TA_BBANDS_Lookback(params[0].as_period(), params[1].as_dev_up(), params[2].as_dev_down(), params[3].as_ma_type()) },
    calculate: bbands_calculate,
});

define_indicator!(RSI, {
    group: Momentum,
    input: Single,
    output: Single,
    description: "Relative Strength Index",
    lookback: |params| unsafe { TA_RSI_Lookback(params[0].as_period()) },
    calculate: rsi_calculate,
});

// 初始化所有指标
pub fn init_indicators() {
    register_sma();
    register_macd();
    register_bbands();
    register_rsi();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicator_engine::talib::indicator_meta::*;

    #[test]
    fn test_new_indicator_system() {
        // 初始化指标
        init_indicators();

        // 测试指标注册表
        let registry = get_indicator_registry();
        let indicators = registry.list_indicators();

        assert!(indicators.contains(&"SMA"));
        assert!(indicators.contains(&"MACD"));
        assert!(indicators.contains(&"BBANDS"));
        assert!(indicators.contains(&"RSI"));

        println!("✅ 指标注册成功: {:?}", indicators);
    }

    #[test]
    fn test_sma_calculation_new_system() {
        init_indicators();

        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let params = vec![IndicatorParam::TimePeriod(5)];
        let input = IndicatorInput::Single(data);

        let result = get_indicator_registry().calculate("SMA", &input, &params);
        assert!(result.is_ok());

        let output = result.unwrap();
        let sma_values = output.as_single().unwrap();

        // 验证SMA计算结果
        assert_eq!(sma_values.len(), 6); // 10 - 5 + 1 = 6
        assert!((sma_values[0] - 3.0).abs() < 0.0001); // (1+2+3+4+5)/5 = 3
        assert!((sma_values[5] - 8.0).abs() < 0.0001); // (6+7+8+9+10)/5 = 8

        println!("✅ SMA计算正确: {:?}", sma_values);
    }

    #[test]
    fn test_indicator_groups() {
        init_indicators();

        let registry = get_indicator_registry();

        // 测试按分组获取指标
        let overlap_indicators = registry.get_indicators_by_group(IndicatorGroup::Overlap);
        assert!(overlap_indicators.contains(&"SMA"));
        assert!(overlap_indicators.contains(&"BBANDS"));

        let momentum_indicators = registry.get_indicators_by_group(IndicatorGroup::Momentum);
        assert!(momentum_indicators.contains(&"MACD"));
        assert!(momentum_indicators.contains(&"RSI"));

        println!("✅ 指标分组功能正常");
        println!("重叠研究指标: {:?}", overlap_indicators);
        println!("动量指标: {:?}", momentum_indicators);
    }

    #[test]
    fn test_indicator_search() {
        init_indicators();

        let registry = get_indicator_registry();

        // 测试搜索功能
        let sma_results = registry.search_indicators("Simple");
        assert!(sma_results.contains(&"SMA"));

        let moving_results = registry.search_indicators("Moving");
        assert!(moving_results.contains(&"SMA"));
        assert!(moving_results.contains(&"MACD"));

        println!("✅ 指标搜索功能正常");
        println!("搜索'Simple': {:?}", sma_results);
        println!("搜索'Moving': {:?}", moving_results);
    }

    #[test]
    fn test_indicator_info() {
        init_indicators();

        let registry = get_indicator_registry();

        // 测试获取指标信息
        let sma_info = registry.get_indicator_info("SMA");
        assert!(sma_info.is_some());

        let (group, input_type, output_format, description) = sma_info.unwrap();
        assert_eq!(*group, IndicatorGroup::Overlap);
        assert_eq!(*input_type, InputType::Single);
        assert_eq!(*output_format, OutputFormat::Single);
        assert_eq!(description, "Simple Moving Average");

        println!("✅ 指标信息获取正常");
        println!("SMA信息: 分组={:?}, 输入={:?}, 输出={:?}, 描述={}",
                 group, input_type, output_format, description);
    }
}

// 为了兼容现有代码，提供转换函数
impl From<IndicatorOutput> for Vec<f64> {
    fn from(output: IndicatorOutput) -> Self {
        match output {
            IndicatorOutput::Single(vec) => vec,
            IndicatorOutput::Dual(vec1, _) => vec1,
            IndicatorOutput::Triple(vec1, _, _) => vec1,
            IndicatorOutput::Quad(vec1, _, _, _) => vec1,
        }
    }
}

impl From<IndicatorOutput> for Vec<Vec<f64>> {
    fn from(output: IndicatorOutput) -> Self {
        match output {
            IndicatorOutput::Single(vec) => {
                // 将单一输出转换为嵌套向量格式
                vec.into_iter().map(|v| vec![v]).collect()
            }
            IndicatorOutput::Dual(vec1, vec2) => {
                let len = vec1.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    result.push(vec![vec1[i], vec2[i]]);
                }
                result
            }
            IndicatorOutput::Triple(vec1, vec2, vec3) => {
                let len = vec1.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    result.push(vec![vec1[i], vec2[i], vec3[i]]);
                }
                result
            }
            IndicatorOutput::Quad(vec1, vec2, vec3, vec4) => {
                let len = vec1.len();
                let mut result = Vec::with_capacity(len);
                for i in 0..len {
                    result.push(vec![vec1[i], vec2[i], vec3[i], vec4[i]]);
                }
                result
            }
        }
    }
}

// 为了向后兼容，保留原有的函数接口
pub fn calculate_sma(data: &[f64], period: i32) -> Result<Vec<f64>, TalibError> {
    let params = vec![IndicatorParam::Period(period)];
    let input = IndicatorInput::Single(data.to_vec());
    let output = get_indicator_registry().calculate("SMA", &input, &params)?;
    Ok(output.into())
}

pub fn calculate_macd(data: &[f64], fast_period: i32, slow_period: i32, signal_period: i32) -> Result<Vec<Vec<f64>>, TalibError> {
    let params = vec![
        IndicatorParam::FastPeriod(fast_period),
        IndicatorParam::SlowPeriod(slow_period),
        IndicatorParam::SignalPeriod(signal_period),
    ];
    let input = IndicatorInput::Single(data.to_vec());
    let output = get_indicator_registry().calculate("MACD", &input, &params)?;
    Ok(output.into())
}

pub fn calculate_bbands(data: &[f64], period: i32, dev_up: f64, dev_down: f64, ma_type: i32) -> Result<Vec<Vec<f64>>, TalibError> {
    let params = vec![
        IndicatorParam::Period(period),
        IndicatorParam::DevUp(dev_up),
        IndicatorParam::DevDown(dev_down),
        IndicatorParam::MAType(ma_type),
    ];
    let input = IndicatorInput::Single(data.to_vec());
    let output = get_indicator_registry().calculate("BBANDS", &input, &params)?;
    Ok(output.into())
}

pub fn calculate_rsi(data: &[f64], period: i32) -> Result<Vec<f64>, TalibError> {
    let params = vec![IndicatorParam::Period(period)];
    let input = IndicatorInput::Single(data.to_vec());
    let output = get_indicator_registry().calculate("RSI", &input, &params)?;
    Ok(output.into())
}

// 通用的指标计算接口
pub fn calculate_indicator_by_name(name: &str, data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    let input = IndicatorInput::Single(data.to_vec());
    get_indicator_registry().calculate(name, &input, params)
}

pub fn get_indicator_lookback(name: &str, params: &[IndicatorParam]) -> Result<i32, TalibError> {
    get_indicator_registry().lookback(name, params)
}

pub fn list_available_indicators() -> Vec<&'static str> {
    get_indicator_registry().list_indicators()
}


