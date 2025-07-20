

#[macro_export]
macro_rules! calculate_fn {
    (
        $indicator_name:ident,
        talib_params => [$(($param_field:ident: $param_type:ty)),* $(,)?],
    ) => {
        paste::paste! {
            pub fn [<calculate_ $indicator_name:lower>](kline_series: Vec<Arc<CacheValue>>, config: &[<$indicator_name Config>]) -> Result<Vec<Indicator>, String> {
                let (timestamp_list, price_source) = CalculateIndicatorFunction::get_price_source_and_timestamp(
                    &config.price_source,
                    kline_series
                )?;

                let result = match TALib::[<$indicator_name:lower>](
                    &timestamp_list,
                    &price_source,
                    $($crate::parse_type!(config, $param_field, $param_type)),*
                ) {
                    Ok(result) => result,
                    Err(e) => return Err(e.to_string()),
                };

                Ok(result)   
            }
        }
    };
}


#[macro_export]
macro_rules! parse_type {
    ($config:expr, $param_field:ident, MAType) => {
        $config.$param_field.clone() as i32
    };
    ($config:expr, $param_field:ident, f64) => {
        $config.$param_field.into_inner()
    };
    ($config:expr, $param_field:ident, f32) => {
        $config.$param_field.into_inner()
    };
    ($config:expr, $param_field:ident, $param_type:ty) => {
        $config.$param_field
    };
}