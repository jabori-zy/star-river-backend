/// 示例：如何使用新的指标系统添加新指标
/// 
/// 这个文件展示了如何快速添加新的技术指标

use crate::indicator_engine::talib::indicator_meta::*;
use crate::indicator_engine::talib_error::TalibError;
use crate::indicator_engine::talib_bindings::*;
use crate::define_indicator;

// 示例1: 添加EMA指标
pub fn ema_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    if params.is_empty() {
        return Err(TalibError::GenericCalculationError {
            error: "EMA requires period parameter".to_string()
        });
    }

    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "EMA requires single input data".to_string()
        }),
    };

    let period = params[0].as_period();

    unsafe {
        let input_size = data.len() as i32;
        let lookback = TA_EMA_Lookback(period);

        if input_size <= lookback {
            return Ok(IndicatorOutput::Single(Vec::new()));
        }

        let expected_out_size = input_size - lookback;
        let mut out: Vec<f64> = vec![0.0; expected_out_size as usize];
        let mut out_begin: i32 = 0;
        let mut out_size: i32 = 0;

        let ret = TA_EMA(
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
                error: format!("TA-Lib EMA error code: {:?}", ret)
            });
        }

        if out_size as usize != expected_out_size as usize {
            out.truncate(out_size as usize);
        }

        Ok(IndicatorOutput::Single(out))
    }
}

define_indicator!(EMA, {
    group: Overlap,
    input: Single,
    output: Single,
    description: "Exponential Moving Average",
    lookback: |params| unsafe { TA_EMA_Lookback(params[0].as_period()) },
    calculate: ema_calculate,
});

// 示例3: 对于更复杂的指标，可以手动定义计算函数
pub fn bollinger_percent_b_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    // 这是一个自定义指标：布林带百分比B
    // %B = (价格 - 下轨) / (上轨 - 下轨)
    
    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "BBPERCENT_B requires single input data".to_string()
        }),
    };

    if params.len() < 3 {
        return Err(TalibError::GenericCalculationError {
            error: "BBPERCENT_B requires 3 parameters: period, dev_up, dev_down".to_string()
        });
    }

    let period = params[0].as_period();
    let dev_up = params[1].as_dev_up();
    let dev_down = params[2].as_dev_down();

    // 首先计算布林带
    let bbands_params = vec![
        IndicatorParam::Period(period),
        IndicatorParam::DevUp(dev_up),
        IndicatorParam::DevDown(dev_down),
        IndicatorParam::MAType(0), // SMA
    ];

    let input_for_bbands = IndicatorInput::Single(data.clone());
    let bbands_result = get_indicator_registry().calculate("BBANDS", &input_for_bbands, &bbands_params)?;
    
    if let IndicatorOutput::Triple(upper_band, _middle_band, lower_band) = bbands_result {
        let mut percent_b = Vec::new();
        let lookback = unsafe { TA_BBANDS_Lookback(period, dev_up, dev_down, 0) } as usize;
        
        for i in 0..upper_band.len() {
            let upper = upper_band[i];
            let lower = lower_band[i];
            let price = data[lookback + i];

            let percent_b_value = if (upper - lower).abs() < f64::EPSILON {
                0.5 // 避免除零
            } else {
                (price - lower) / (upper - lower)
            };

            percent_b.push(percent_b_value);
        }
        
        Ok(IndicatorOutput::Single(percent_b))
    } else {
        Err(TalibError::GenericCalculationError { 
            error: "Failed to calculate BBands for %B".to_string() 
        })
    }
}

// 手动定义复杂指标
define_indicator!(BBPERCENT_B, {
    group: Momentum,
    input: Single,
    output: Single,
    description: "Bollinger Bands %B",
    lookback: |params| unsafe { TA_BBANDS_Lookback(params[0].as_period(), params[1].as_dev_up(), params[2].as_dev_down(), 0) },
    calculate: bollinger_percent_b_calculate,
});

// 示例4: 组合指标 - 双移动平均线交叉信号
pub fn dual_ma_cross_calculate(input: &IndicatorInput, params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
    let data = match input {
        IndicatorInput::Single(data) => data,
        _ => return Err(TalibError::GenericCalculationError {
            error: "DUAL_MA_CROSS requires single input data".to_string()
        }),
    };

    if params.len() < 2 {
        return Err(TalibError::GenericCalculationError {
            error: "DUAL_MA_CROSS requires 2 parameters: fast_period, slow_period".to_string()
        });
    }

    let fast_period = params[0].as_fast_period();
    let slow_period = params[1].as_slow_period();

    // 计算快速MA
    let fast_ma_params = vec![IndicatorParam::Period(fast_period)];
    let input_for_ma = IndicatorInput::Single(data.clone());
    let fast_ma = get_indicator_registry().calculate("SMA", &input_for_ma, &fast_ma_params)?;

    // 计算慢速MA
    let slow_ma_params = vec![IndicatorParam::Period(slow_period)];
    let slow_ma = get_indicator_registry().calculate("SMA", &input_for_ma, &slow_ma_params)?;
    
    if let (IndicatorOutput::Single(fast_values), IndicatorOutput::Single(slow_values)) = (fast_ma, slow_ma) {
        let mut signals = Vec::new();
        let min_len = fast_values.len().min(slow_values.len());
        
        for i in 0..min_len {
            let signal = if fast_values[i] > slow_values[i] {
                1.0  // 金叉信号
            } else if fast_values[i] < slow_values[i] {
                -1.0 // 死叉信号
            } else {
                0.0  // 无信号
            };
            signals.push(signal);
        }
        
        Ok(IndicatorOutput::Single(signals))
    } else {
        Err(TalibError::GenericCalculationError { 
            error: "Failed to calculate MA values for cross signal".to_string() 
        })
    }
}

define_indicator!(DUAL_MA_CROSS, {
    group: Momentum,
    input: Single,
    output: Single,
    description: "Dual Moving Average Crossover Signal",
    lookback: |params| {
        let fast_period = params[0].as_fast_period();
        let slow_period = params[1].as_slow_period();
        unsafe { TA_SMA_Lookback(slow_period.max(fast_period)) }
    },
    calculate: dual_ma_cross_calculate,
});

// 初始化示例指标
pub fn init_example_indicators() {
    register_ema();
    register_bbpercent_b();
    register_dual_ma_cross();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_indicators() {
        // 初始化基础指标和示例指标
        crate::indicator_engine::talib::indicators::init_indicators();
        init_example_indicators();
        
        let indicators = get_indicator_registry().list_indicators();
        assert!(indicators.contains(&"EMA"));
        assert!(indicators.contains(&"BBPERCENT_B"));
        assert!(indicators.contains(&"DUAL_MA_CROSS"));
    }

    #[test]
    fn test_dual_ma_cross() {
        crate::indicator_engine::talib::indicators::init_indicators();
        init_example_indicators();
        
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0];
        let params = vec![
            IndicatorParam::FastPeriod(3),
            IndicatorParam::SlowPeriod(5),
        ];
        
        let input = IndicatorInput::Single(data);
        let result = get_indicator_registry().calculate("DUAL_MA_CROSS", &input, &params).unwrap();
        
        match result {
            IndicatorOutput::Single(signals) => {
                assert!(!signals.is_empty());
                // 在上升趋势中应该有金叉信号
                assert!(signals.iter().any(|&s| s > 0.0));
            }
            _ => panic!("Expected single output for DUAL_MA_CROSS"),
        }
    }
}

/*
使用指南：

1. 对于标准的TA-Lib指标，使用simple_indicator!宏：
   ```rust
   simple_indicator!(INDICATOR_NAME, output_type, TA_FUNCTION_NAME, ERROR_TYPE, param1, param2, ...);
   ```

2. 对于自定义指标，定义计算函数然后使用define_indicator!宏：
   ```rust
   pub fn my_indicator_calculate(data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
       // 自定义计算逻辑
   }

   define_indicator!(MY_INDICATOR, {
       lookback: |params| { /* 计算lookback */ },
       calculate: my_indicator_calculate,
       output: Single, // 或 Triple
   });
   ```

3. 在indicators.rs的init_indicators()函数中调用register_xxx()函数

4. 新指标会自动支持通用接口：
   - TALib::calculate_indicator()
   - TALib::get_indicator_lookback()
   - TALib::list_indicators()
*/
