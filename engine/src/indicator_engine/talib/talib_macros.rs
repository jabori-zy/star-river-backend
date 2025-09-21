#[macro_export]
macro_rules! talib_fn {
    (
        $indicator_name:ident,
        datetime => ($datetime_field:ident: $datetime_type:ty),
        input => [($input_field:ident: $input_type:ty)], // 单个输入
        talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
        output => [$(($output_name:ident: $output_type:ty)),* $(,)?],
    ) => {
        paste::paste! {
            pub fn [<$indicator_name:lower>]($datetime_field: $datetime_type, $input_field: $input_type, $($param_field: $param_type),*) -> Result<Vec<Indicator>, IndicatorEngineError> {
                let input_size = $input_field.len();
                let lookback = unsafe {
                    [<TA_ $indicator_name:upper _Lookback>]
                    (
                        $($param_field),*
                    ) as usize
                };
                if input_size <= lookback {
                    return Err(DataLessThenLookbackSnafu {
                        indicator_name: stringify!($indicator_name).to_string(),
                        lookback,
                        data_length: input_size,
                    }.build());
                }

                let ($([<out_ $output_name>],)*) = crate::execute_talib_function!(
                    $indicator_name,
                    input_size,
                    lookback,
                    [$input_field],
                    [$($param_field),*],
                    [$($output_name: $output_type),*]
                );

                let result: Vec<Indicator> = (0..input_size)
                    .map(|i| $indicator_name {
                        datetime: $datetime_field[i],
                        $(
                            $output_name: [<out_ $output_name>][i],
                        )*
                    }.into())
                    .collect();

                Ok(result)
            }
        }
    };

    (
        $indicator_name:ident,
        datetime => ($datetime_field:ident: $datetime_type:ty),
        input => [$(($input_field:ident: $input_type:ty)),* $(,)?], // 多个输入
        talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
        output => [$(($output_name:ident: $output_type:ty)),* $(,)?],
    ) => {
        paste::paste! {
            pub fn [<$indicator_name:lower>](
                $datetime_field: $datetime_type,
                $($input_field: $input_type,)*
                $($param_field: $param_type),*
            ) -> Result<Vec<Indicator>, IndicatorEngineError> {

                let mut input_size = Vec::new();
                // 计算每一个输入的长度
                $(
                    input_size.push($input_field.len());
                )*

                // 判断列表每一个元素是否相等
                let first_size = &input_size[0];
                for size in &input_size {
                    if size != first_size {
                        return Err(DataLengthNotEqualSnafu {
                            data_length: input_size.clone(),
                        }.build());
                    }
                }

                // 如果相等，则取第一个值，用来和lookback比较
                let input_size = input_size[0];

                let lookback = unsafe {
                    [<TA_ $indicator_name:upper _Lookback>]
                    (
                        $($param_field),*
                    ) as usize
                };

                if input_size <= lookback {
                    return Err(DataLessThenLookbackSnafu {
                        indicator_name: stringify!($indicator_name).to_string(),
                        lookback,
                        data_length: input_size,
                    }.build());
                }

                let ($([<out_ $output_name>],)*) = crate::execute_talib_function!(
                    $indicator_name,
                    input_size,
                    lookback,
                    [$($input_field),*],
                    [$($param_field),*],
                    [$($output_name: $output_type),*]
                );

                let result: Vec<Indicator> = (0..input_size)
                    .map(|i| $indicator_name {
                        datetime: $datetime_field[i],
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

#[macro_export]
macro_rules! talib_snake_fn {
    (
        $indicator_name:ident,
        datetime => ($datetime_field:ident: $datetime_type:ty),
        input => [($input_field:ident: $input_type:ty)],
        talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
        output => [$(($output_name:ident: $output_type:ty)),* $(,)?],
    ) => {
        paste::paste! {
            pub fn [<$indicator_name:snake:lower>]($datetime_field: $datetime_type, $input_field: $input_type, $($param_field: $param_type),*) -> Result<Vec<Indicator>, IndicatorEngineError> {
                let input_size = $input_field.len();
                let lookback = unsafe {
                    [<TA_ $indicator_name:snake:upper _Lookback>]
                    (
                        $($param_field),*
                    ) as usize
                };
                if input_size <= lookback {
                    return Err(DataLessThenLookbackSnafu {
                        indicator_name: stringify!($indicator_name).to_string(),
                        lookback,
                        data_length: input_size,
                    }.build());
                }

                let ($([<out_ $output_name>],)*) = crate::execute_talib_function_snake!(
                    $indicator_name,
                    input_size,
                    lookback,
                    [$input_field],
                    [$($param_field),*],
                    [$($output_name: $output_type),*]
                );

                let result: Vec<Indicator> = (0..input_size)
                    .map(|i| $indicator_name {
                        datetime: $datetime_field[i],
                        $(
                            $output_name: [<out_ $output_name>][i],
                        )*
                    }.into())
                    .collect();

                Ok(result)
            }
        }
    };

    (
        $indicator_name:ident,
        datetime => ($datetime_field:ident: $datetime_type:ty),
        input => [$(($input_field:ident: $input_type:ty)),* $(,)?],
        talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
        output => [$(($output_name:ident: $output_type:ty)),* $(,)?],
    ) => {
        paste::paste! {
            pub fn [<$indicator_name:snake:lower>](
                $datetime_field: $datetime_type,
                $($input_field: $input_type,)*
                $($param_field: $param_type),*
            ) -> Result<Vec<Indicator>, IndicatorEngineError> {

                let mut input_size = Vec::new();
                // 计算每一个输入的长度
                $(
                    input_size.push($input_field.len());
                )*

                // 判断列表每一个元素是否相等
                let first_size = &input_size[0];
                for size in &input_size {
                    if size != first_size {
                        return Err(DataLengthNotEqualSnafu {
                            data_length: input_size.clone(),
                        }.build());
                    }
                }

                // 如果相等，则取第一个值，用来和lookback比较
                let input_size = input_size[0];

                let lookback = unsafe {
                    [<TA_ $indicator_name:snake:upper _Lookback>]
                    (
                        $($param_field),*
                    ) as usize
                };

                if input_size <= lookback {
                    return Err(DataLessThenLookbackSnafu {
                        indicator_name: stringify!($indicator_name).to_string(),
                        lookback,
                        data_length: input_size,
                    }.build());
                }

                let ($([<out_ $output_name>],)*) = crate::execute_talib_function_snake!(
                    $indicator_name,
                    input_size,
                    lookback,
                    [$($input_field),*],
                    [$($param_field),*],
                    [$($output_name: $output_type),*]
                );

                let result: Vec<Indicator> = (0..input_size)
                    .map(|i| $indicator_name {
                        datetime: $datetime_field[i],
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

// 内部宏：处理 TA-Lib 函数调用和错误处理的通用逻辑
#[macro_export]
macro_rules! execute_talib_function {
    (
        $indicator_name:ident,
        $input_size:expr,
        $lookback:expr,
        [$($input_field:ident),*],
        [$($param_field:ident),*],
        [$($output_name:ident: $output_type:ty),*]
    ) => {{
        paste::paste! {
            let mut out_begin_index = 0;
            let mut out_number_elements = 0;
            $(
                let mut [<out_ $output_name>]: Vec<$output_type> = vec![$output_type::default(); $input_size];
            )*

            let ret = unsafe {
                [<TA_ $indicator_name:upper>]
                (
                    0,
                    ($input_size - 1) as i32,
                    $(
                        $input_field.as_ptr(),
                    )*
                    $($param_field,)*
                    &mut out_begin_index,
                    &mut out_number_elements,
                    $(
                        [<out_ $output_name>].as_mut_ptr().add($lookback),
                    )*
                )
            };

            if ret != TA_RetCode_TA_SUCCESS {
                return Err(TalibSnafu {
                    ret_code: ret,
                }.build());
            }

            ($(([<out_ $output_name>]),)*)
        }
    }};
}

#[macro_export]
macro_rules! execute_talib_function_snake {
    (
        $indicator_name:ident,
        $input_size:expr,
        $lookback:expr,
        [$($input_field:ident),*],
        [$($param_field:ident),*],
        [$($output_name:ident: $output_type:ty),*]
    ) => {{
        paste::paste! {
            let mut out_begin_index = 0;
            let mut out_number_elements = 0;
            $(
                let mut [<out_ $output_name>]: Vec<$output_type> = vec![$output_type::default(); $input_size];
            )*

            let ret = unsafe {
                [<TA_ $indicator_name:snake:upper>]
                (
                    0,
                    ($input_size - 1) as i32,
                    $(
                        $input_field.as_ptr(),
                    )*
                    $($param_field,)*
                    &mut out_begin_index,
                    &mut out_number_elements,
                    $(
                        [<out_ $output_name>].as_mut_ptr().add($lookback),
                    )*
                )
            };

            if ret != TA_RetCode_TA_SUCCESS {
                return Err(TalibSnafu {
                    ret_code: ret,
                }.build());
            }

            ($(([<out_ $output_name>]),)*)
        }
    }};
}
