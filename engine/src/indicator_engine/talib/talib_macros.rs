#[macro_export]
macro_rules! talib_fn {
    // // 单个输出的情况
    // (
    //     $indicator_name:ident,
    //     timestamp => ($timestamp_field:ident: $timestamp_type:ty),
    //     input => [($input_field:ident: $input_type:ty)],
    //     talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
    //     output => [$output_name:ident],
    // ) => {
    //     paste::paste! {
    //         pub fn [<$indicator_name:lower>]($timestamp_field: $timestamp_type, $input_field: $input_type, $($param_field: $param_type),*) -> Result<Vec<Indicator>, TalibError> {
    //             let input_size = $input_field.len();
    //             let lookback = unsafe { [<TA_ $indicator_name:upper _Lookback>]($($param_field),*) as usize };
                
    //             let mut out_begin_index = 0;
    //             let mut out_number_elements = 0;
    //             let mut [<out_ $output_name>]: Vec<f64> = vec![0.0; input_size];

    //             let ret = unsafe {
    //                 [<TA_ $indicator_name:upper>](
    //                     0,
    //                     (input_size - 1) as i32,
    //                     $input_field.as_ptr(),
    //                     $($param_field),*,
    //                     &mut out_begin_index,
    //                     &mut out_number_elements,
    //                     [<out_ $output_name>].as_mut_ptr().add(lookback),
    //                 )
    //             };

    //             if ret != TA_RetCode_TA_SUCCESS {
    //                 return Err(TalibError::GenericCalculationError {
    //                     error: format!("TA-Lib error code: {:?}", ret)
    //                 });
    //             }
                
    //             let result: Vec<Indicator> = (0..input_size)
    //                 .map(|i| $indicator_name {
    //                     timestamp: $timestamp_field[i],
    //                     $output_name: [<out_ $output_name>][i],
    //                 }.into())
    //                 .collect();
                
    //             Ok(result)
    //         }
    //     }
    // };
    
    // 多个输出的情况
    (
        $indicator_name:ident,
        timestamp => ($timestamp_field:ident: $timestamp_type:ty),
        input => [($input_field:ident: $input_type:ty)],
        talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
        output => [$($output_name:ident),+ $(,)?],
    ) => {
        paste::paste! {
            pub fn [<$indicator_name:lower>]($timestamp_field: $timestamp_type, $input_field: $input_type, $($param_field: $param_type),*) -> Result<Vec<Indicator>, TalibError> {
                let input_size = $input_field.len();
                let lookback = unsafe { [<TA_ $indicator_name:upper _Lookback>]($($param_field),*) as usize };
                if input_size <= lookback {
                    return Err(TalibError::DataLessThenLookbackError {
                        indicator_name: stringify!($indicator_name).to_string(),
                        lookback,
                        data_length: input_size,
                    });
                }
                
                let mut out_begin_index = 0;
                let mut out_number_elements = 0;
                $(
                    let mut [<out_ $output_name>]: Vec<f64> = vec![0.0; input_size];
                )*

                let ret = unsafe {
                    [<TA_ $indicator_name:upper>](
                        0,
                        (input_size - 1) as i32,
                        $input_field.as_ptr(),
                        $($param_field),*,
                        &mut out_begin_index,
                        &mut out_number_elements,
                        $(
                            [<out_ $output_name>].as_mut_ptr().add(lookback),
                        )*
                    )
                };

                if ret != TA_RetCode_TA_SUCCESS {
                    return Err(TalibError::GenericCalculationError {
                        error: format!("TA-Lib error code: {:?}", ret)
                    });
                }
                
                let result: Vec<Indicator> = (0..input_size)
                    .map(|i| $indicator_name {
                        timestamp: $timestamp_field[i],
                        $(
                            $output_name: [<out_ $output_name>][i],
                        )*
                    }.into())
                    .collect();
                
                Ok(result)
            }
        }
    };
}

