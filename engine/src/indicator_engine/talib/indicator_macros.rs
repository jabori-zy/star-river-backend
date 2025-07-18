/// 定义指标的宏
///
/// 使用方式:
/// ```rust
/// define_indicator!(SMA, {
///     group: Overlap,
///     input: Single,
///     output: Single,
///     description: "Simple Moving Average",
///     lookback: |params| unsafe { TA_SMA_Lookback(params[0].as_period()) },
///     calculate: sma_calculate,
/// });
/// ```
#[macro_export]
macro_rules! define_indicator {
    ($name:ident, {
        group: $group:ident,
        input: $input_type:ident,
        output: $output_format:ident,
        description: $description:expr,
        lookback: $lookback_fn:expr,
        calculate: $calculate_fn:ident,
    }) => {
        paste::paste! {
            pub fn [<register_ $name:lower>]() {
                use crate::indicator_engine::talib::indicator_meta::*;

                let meta = IndicatorMeta {
                    name: stringify!($name),
                    group: IndicatorGroup::$group,
                    input_type: InputType::$input_type,
                    output_format: OutputFormat::$output_format,
                    description: $description,
                    lookback_fn: $lookback_fn,
                    calculate_fn: $calculate_fn,
                };

                get_indicator_registry_mut().register(meta);
            }
        }
    };
}

/// 生成标准的单输出指标计算函数
#[macro_export]
macro_rules! generate_single_output_calculator {
    ($fn_name:ident, $ta_fn:ident, $error_variant:ident, $($param:ident),*) => {
        pub fn $fn_name(data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
            use crate::indicator_engine::talib_bindings::*;
            use crate::indicator_engine::talib::indicator_meta::*;
            
            unsafe {
                let input_size = data.len() as i32;
                
                // 提取参数
                let mut param_iter = params.iter();
                $(
                    let $param = param_iter.next()
                        .ok_or_else(|| TalibError::GenericCalculationError {
                            error: format!("Missing parameter: {}", stringify!($param))
                        })?;
                )*
                
                // 计算lookback
                let lookback = paste::paste! { [<TA_ $ta_fn _Lookback>] }(
                    $($param.as_period()),*
                );
                
                // 检查数据长度
                if input_size <= lookback {
                    return Ok(IndicatorOutput::Single(Vec::new()));
                }
                
                let expected_out_size = input_size - lookback;
                let mut out: Vec<f64> = vec![0.0; expected_out_size as usize];
                let mut out_begin: i32 = 0;
                let mut out_size: i32 = 0;
                
                // 调用TA-Lib函数
                let ret = paste::paste! { [<TA_ $ta_fn>] }(
                    0,
                    input_size - 1,
                    data.as_ptr(),
                    $($param.as_period()),*
                    &mut out_begin,
                    &mut out_size,
                    out.as_mut_ptr(),
                );
                
                if ret != TA_RetCode_TA_SUCCESS {
                    return Err(TalibError::GenericCalculationError {
                        error: format!("TA-Lib error code: {:?}", ret)
                    });
                }
                
                // 调整输出大小
                if out_size as usize != expected_out_size as usize {
                    out.truncate(out_size as usize);
                }
                
                Ok(IndicatorOutput::Single(out))
            }
        }
    };
}

/// 生成标准的三重输出指标计算函数
#[macro_export]
macro_rules! generate_triple_output_calculator {
    ($fn_name:ident, $ta_fn:ident, $error_variant:ident, $($param:ident),*) => {
        pub fn $fn_name(data: &[f64], params: &[IndicatorParam]) -> Result<IndicatorOutput, TalibError> {
            use crate::indicator_engine::talib_bindings::*;
            use crate::indicator_engine::talib::indicator_meta::*;
            
            unsafe {
                let input_size = data.len() as i32;
                
                // 提取参数
                let mut param_iter = params.iter();
                $(
                    let $param = param_iter.next()
                        .ok_or_else(|| TalibError::GenericCalculationError {
                            error: format!("Missing parameter: {}", stringify!($param))
                        })?;
                )*
                
                // 计算lookback
                let lookback = paste::paste! { [<TA_ $ta_fn _Lookback>] }(
                    $($param.as_period()),*
                );
                
                // 检查数据长度
                if input_size <= lookback {
                    return Ok(IndicatorOutput::Triple(Vec::new()));
                }
                
                let expected_out_size = input_size - lookback;
                let mut out1: Vec<f64> = vec![0.0; expected_out_size as usize];
                let mut out2: Vec<f64> = vec![0.0; expected_out_size as usize];
                let mut out3: Vec<f64> = vec![0.0; expected_out_size as usize];
                let mut out_begin: i32 = 0;
                let mut out_size: i32 = 0;
                
                // 调用TA-Lib函数
                let ret = paste::paste! { [<TA_ $ta_fn>] }(
                    0,
                    input_size - 1,
                    data.as_ptr(),
                    $($param.[<as_ $param:lower>]()),*
                    &mut out_begin,
                    &mut out_size,
                    out1.as_mut_ptr(),
                    out2.as_mut_ptr(),
                    out3.as_mut_ptr(),
                );
                
                if ret != TA_RetCode_TA_SUCCESS {
                    return Err(TalibError::$error_variant { 
                        error: format!("TA-Lib error code: {:?}", ret)
                    });
                }
                
                // 调整输出大小并组合结果
                if out_size as usize != expected_out_size as usize {
                    out1.truncate(out_size as usize);
                    out2.truncate(out_size as usize);
                    out3.truncate(out_size as usize);
                }
                
                let mut result = Vec::with_capacity(out_size as usize);
                for i in 0..out_size as usize {
                    result.push(vec![out1[i], out2[i], out3[i]]);
                }
                
                Ok(IndicatorOutput::Triple(result))
            }
        }
    };
}

/// 简化的指标定义宏，自动生成计算函数
#[macro_export]
macro_rules! simple_indicator {
    ($name:ident, single, $ta_fn:ident, $error_variant:ident, $($param:ident),*) => {
        paste::paste! {
            generate_single_output_calculator!([<$name:lower _calculate>], $ta_fn, $error_variant, $($param),*);
            
            define_indicator!($name, {
                lookback: |params| {
                    let mut param_iter = params.iter();
                    $(
                        let $param = param_iter.next().unwrap();
                    )*
                    unsafe { 
                        paste::paste! { [<TA_ $ta_fn _Lookback>] }($($param.[<as_ $param:lower>]()),*)
                    }
                },
                calculate: [<$name:lower _calculate>],
                output: Single,
            });
        }
    };
    
    ($name:ident, triple, $ta_fn:ident, $error_variant:ident, $($param:ident),*) => {
        paste::paste! {
            generate_triple_output_calculator!([<$name:lower _calculate>], $ta_fn, $error_variant, $($param),*);
            
            define_indicator!($name, {
                lookback: |params| {
                    let mut param_iter = params.iter();
                    $(
                        let $param = param_iter.next().unwrap();
                    )*
                    unsafe { 
                        paste::paste! { [<TA_ $ta_fn _Lookback>] }($($param.[<as_ $param:lower>]()),*)
                    }
                },
                calculate: [<$name:lower _calculate>],
                output: Triple,
            });
        }
    };
}
