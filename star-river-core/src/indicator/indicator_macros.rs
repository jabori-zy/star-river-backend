// 辅助函数：将下划线命名转换为驼峰命名
pub fn snake_to_camel(snake_str: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for ch in snake_str.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[macro_export]
macro_rules! define_indicator_output {
    (
        $indicator_name:ident,
        output => [$(($output_field:ident: $output_type:ty)),* $(,)?] $(,)?
    ) => {
        paste::paste! {
            // 生成输出结构体
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, deepsize::DeepSizeOf)]
            pub struct $indicator_name {
                $(
                    pub $output_field: $output_type,
                )*
            }

            impl From<$indicator_name> for crate::indicator::Indicator {
                fn from(indicator: $indicator_name) -> Self {
                    crate::indicator::Indicator::$indicator_name(indicator)
                }
            }

            // impl From<$indicator_name> for crate::cache::CacheValue {
            //     fn from(indicator: $indicator_name) -> Self {
            //         crate::cache::CacheValue::Indicator(indicator.into())
            //     }
            // }

            impl crate::market::QuantData for $indicator_name {
                fn get_datetime(&self) -> crate::system::DateTimeUtc {
                    self.datetime
                }
                fn to_json(&self) -> serde_json::Value {
                    serde_json::to_value(self).unwrap()
                }

                fn to_list(&self) -> Vec<f64> {
                    let mut result = Vec::new();
                    $(
                        $crate::add_field_to_vec_dispatch!(result, self.$output_field);
                    )*
                    result
                }

                fn to_json_with_time(&self) -> serde_json::Value {
                    use serde_json::json;
                    json!({
                        $(
                            stringify!($output_field): $crate::format_field_with_time!(self.$output_field, stringify!($output_field), $output_type),
                        )*
                    })
                }

                fn get_value(&self, key: &str) -> Option<f64> {
                    match key {
                        $(
                            stringify!($output_field) => $crate::get_field_value_dispatch!(self.$output_field),
                        )*
                        _ => None,
                    }
                }
            }

            // impl $indicator_name {

            //     fn get_datetime(&self) -> DateTime<Utc> {
            //         self.datetime
            //     }

            //     fn to_json(&self) -> serde_json::Value {
            //         crate::market::QuantData::to_json(self)
            //     }

            //     fn to_list(&self) -> Vec<f64> {
            //         crate::market::QuantData::to_list(self)
            //     }

            //     fn get_timestamp(&self) -> i64 {
            //         self.datetime.timestamp_millis()
            //     }

            //     fn to_json_with_time(&self) -> serde_json::Value {
            //         crate::market::QuantData::to_json_with_time(self)
            //     }
            // }
        }
    };
}

#[macro_export]
macro_rules! define_indicator_config {
    (
        $indicator_name:ident,// 传入字符串
        params => []
    ) => {
        paste::paste! {
            // 生成配置结构体
            #[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            pub struct [<$indicator_name Config>];

            impl ToString for [<$indicator_name Config>] {
                fn to_string(&self) -> String {
                    {
                        const NAME_STR: &str = stringify!($indicator_name);
                        let snake_name = if NAME_STR.chars().all(|c| c.is_uppercase() || c.is_numeric()) {
                            NAME_STR.to_lowercase()
                        } else {
                            let mut result = String::new();
                            for (i, ch) in NAME_STR.char_indices() {
                                if ch.is_uppercase() && i > 0 {
                                    result.push('_');
                                }
                                result.push(ch.to_ascii_lowercase());
                            }
                            result
                        };
                        format!("{}()", snake_name)
                    }
                }
            }

            impl std::str::FromStr for [<$indicator_name Config>] {
                type Err = StarRiverError;
                #[allow(unused_variables)]
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    // use crate::indicator::utils::*;
                    // let (_name, params) = parse_indicator_config_from_str(s)?;
                    Ok(Self {})
                }
            }

            impl crate::indicator::IndicatorConfigTrait for [<$indicator_name Config>] {
                #[allow(unused_variables)]
                fn new(config: &serde_json::Value) -> Result<Self, serde_json::Error> {
                    Ok(Self {})
                }
            }
        }
    };

    (
        $indicator_name:ident,// 传入字符串
        params => [$(($param:ident: $param_type:ty)),* $(,)?] $(,)?
    ) => {
        paste::paste! {
            // 生成配置结构体
            #[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct [<$indicator_name Config>] {
                $(
                    pub $param: $crate::wrap_float_type!($param_type),
                )*
            }

            impl ToString for [<$indicator_name Config>] {
                fn to_string(&self) -> String {
                    let mut params: Vec<String> = Vec::new();
                    $(
                        params.push(format!("{}={}", stringify!($param), self.$param.to_string()));
                    )*
                    {
                        const NAME_STR: &str = stringify!($indicator_name);
                        let snake_name = if NAME_STR.chars().all(|c| c.is_uppercase() || c.is_numeric()) {
                            NAME_STR.to_lowercase()
                        } else {
                            let mut result = String::new();
                            for (i, ch) in NAME_STR.char_indices() {
                                if ch.is_uppercase() && i > 0 {
                                    result.push('_');
                                }
                                result.push(ch.to_ascii_lowercase());
                            }
                            result
                        };
                        format!("{}({})", snake_name, params.join(" "))
                    }
                }
            }

            impl std::str::FromStr for [<$indicator_name Config>] {
                type Err = StarRiverError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    use crate::indicator::utils::*;
                    let (_name, params) = parse_indicator_config_from_str(s)?;
                    $(
                        let $param = $crate::parse_param_by_type!(params, stringify!($param), $param_type);
                    )*
                    Ok(Self {
                        $($param),*
                    })
                }
            }

            impl crate::indicator::IndicatorConfigTrait for [<$indicator_name Config>] {
                fn new(config: &serde_json::Value) -> Result<Self, serde_json::Error> {
                    // 直接使用 serde_json 反序列化，它会自动处理驼峰命名转换
                    let config = serde_json::from_value(config.clone())?;
                    Ok(config)
                }
            }
        }
    };
}

// 辅助宏：包装浮点类型为OrderedFloat
#[macro_export]
macro_rules! wrap_float_type {
    (f64) => {
        ordered_float::OrderedFloat<f64>
    };
    (f32) => {
        ordered_float::OrderedFloat<f32>
    };
    ($other_type:ty) => {
        $other_type
    };
}

// 辅助宏：将字段添加到Vec中（用于to_list方法）
#[macro_export]
macro_rules! add_field_to_vec {
    ($vec:expr, $field:expr, i64) => {
        $vec.push($field as f64);
    };
    ($vec:expr, $field:expr, i32) => {
        $vec.push($field as f64);
    };
    ($vec:expr, $field:expr, f64) => {
        $vec.push($field);
    };
    ($vec:expr, $field:expr, f32) => {
        $vec.push($field as f64);
    };
    ($vec:expr, $field:expr, ordered_float::OrderedFloat<f64>) => {
        $vec.push($field.into_inner());
    };
    ($vec:expr, $field:expr, ordered_float::OrderedFloat<f32>) => {
        $vec.push($field.into_inner() as f64);
    };
    ($vec:expr, $field:expr, chrono::DateTime<chrono::FixedOffset>) => {
        $vec.push($field.timestamp_millis() as f64);
    };
    ($vec:expr, $field:expr, chrono::DateTime<Utc>) => {
        $vec.push($field.timestamp_millis() as f64);
    };
    ($vec:expr, $field:expr, crate::time::Utc8DateTime) => {
        $vec.push($field.timestamp_millis() as f64);
    };
    ($vec:expr, $field:expr, DateTime<Utc>) => {
        $vec.push($field.timestamp_millis() as f64);
    };
}

// 辅助宏：通过类型推断将字段添加到Vec中
#[macro_export]
macro_rules! add_field_to_vec_dispatch {
    ($vec:expr, $field:expr, Option<$inner:ty>) => {
        match &$field {
            Some(inner) => {
                let inner = inner.clone();
                $crate::add_field_to_vec!($vec, inner, $inner);
            }
            None => $vec.push(f64::NAN),
        }
    };
    ($vec:expr, $field:expr) => {
        // 使用匿名函数来处理不同类型的转换
        let convert_value = |field: &dyn std::any::Any| -> f64 {
            if let Some(val) = field.downcast_ref::<i64>() {
                *val as f64
            } else if let Some(val) = field.downcast_ref::<i32>() {
                *val as f64
            } else if let Some(val) = field.downcast_ref::<f64>() {
                *val
            } else if let Some(val) = field.downcast_ref::<f32>() {
                *val as f64
            } else if let Some(val) = field.downcast_ref::<ordered_float::OrderedFloat<f64>>() {
                val.into_inner()
            } else if let Some(val) = field.downcast_ref::<ordered_float::OrderedFloat<f32>>() {
                val.into_inner() as f64
            } else if let Some(val) = field.downcast_ref::<Option<f64>>() {
                val.unwrap_or(f64::NAN)
            } else if let Some(val) = field.downcast_ref::<Option<f32>>() {
                val.map(|v| v as f64).unwrap_or(f64::NAN)
            } else if let Some(val) = field.downcast_ref::<Option<i64>>() {
                val.map(|v| v as f64).unwrap_or(f64::NAN)
            } else if let Some(val) = field.downcast_ref::<Option<i32>>() {
                val.map(|v| v as f64).unwrap_or(f64::NAN)
            } else if let Some(val) = field.downcast_ref::<Option<ordered_float::OrderedFloat<f64>>>() {
                val.map(|v| v.into_inner()).unwrap_or(f64::NAN)
            } else if let Some(val) = field.downcast_ref::<Option<ordered_float::OrderedFloat<f32>>>() {
                val.map(|v| v.into_inner() as f64).unwrap_or(f64::NAN)
            } else if let Some(val) = field.downcast_ref::<chrono::DateTime<chrono::FixedOffset>>() {
                val.timestamp_millis() as f64
            } else if let Some(val) = field.downcast_ref::<chrono::DateTime<chrono::Utc>>() {
                val.timestamp_millis() as f64
            } else if let Some(val) = field.downcast_ref::<crate::system::DateTimeUtc>() {
                val.timestamp_millis() as f64
            } else {
                // 对于无法识别的类型，使用NaN而不是0.0来表示无效值
                f64::NAN
            }
        };
        $vec.push(convert_value(&$field));
    };
}

// 辅助宏：通过类型推断将字段转换为Option<f64>（用于get_value方法）
#[macro_export]
macro_rules! get_field_value_dispatch {
    ($field:expr) => {
        // 使用匿名函数来处理不同类型的转换
        (|field: &dyn std::any::Any| -> Option<f64> {
            if let Some(val) = field.downcast_ref::<i64>() {
                Some(*val as f64)
            } else if let Some(val) = field.downcast_ref::<i32>() {
                Some(*val as f64)
            } else if let Some(val) = field.downcast_ref::<f64>() {
                Some(*val)
            } else if let Some(val) = field.downcast_ref::<f32>() {
                Some(*val as f64)
            } else if let Some(val) = field.downcast_ref::<ordered_float::OrderedFloat<f64>>() {
                Some(val.into_inner())
            } else if let Some(val) = field.downcast_ref::<ordered_float::OrderedFloat<f32>>() {
                Some(val.into_inner() as f64)
            } else if let Some(val) = field.downcast_ref::<Option<f64>>() {
                *val
            } else if let Some(val) = field.downcast_ref::<Option<f32>>() {
                val.map(|v| v as f64)
            } else if let Some(val) = field.downcast_ref::<Option<i64>>() {
                val.map(|v| v as f64)
            } else if let Some(val) = field.downcast_ref::<Option<i32>>() {
                val.map(|v| v as f64)
            } else if let Some(val) = field.downcast_ref::<Option<ordered_float::OrderedFloat<f64>>>() {
                val.map(|v| v.into_inner())
            } else if let Some(val) = field.downcast_ref::<Option<ordered_float::OrderedFloat<f32>>>() {
                val.map(|v| v.into_inner() as f64)
            } else if let Some(val) = field.downcast_ref::<chrono::DateTime<chrono::FixedOffset>>() {
                Some(val.timestamp_millis() as f64)
            } else if let Some(val) = field.downcast_ref::<chrono::DateTime<chrono::Utc>>() {
                Some(val.timestamp_millis() as f64)
            } else if let Some(val) = field.downcast_ref::<crate::system::DateTimeUtc>() {
                Some(val.timestamp_millis() as f64)
            } else {
                // 对于无法识别的类型，返回None
                None
            }
        })(&$field)
    };
}

// 辅助宏：格式化字段（用于to_json_with_time方法）
#[macro_export]
macro_rules! format_field_with_time {
    ($field:expr, $field_name:expr, i64) => {
        if $field_name == "timestamp" {
            serde_json::Value::String(crate::utils::timestamp_to_utc8($field))
        } else {
            serde_json::Value::from($field)
        }
    };
    ($field:expr, $field_name:expr, chrono::DateTime<chrono::FixedOffset>) => {
        serde_json::Value::String($field.to_string())
    };
    ($field:expr, $field_name:expr, crate::time::Utc8DateTime) => {
        serde_json::Value::String($field.to_string())
    };
    ($field:expr, $field_name:expr, DateTimeUtc) => {
        serde_json::Value::String($field.to_string())
    };
    ($field:expr, $field_name:expr, $other_type:ty) => {
        serde_json::to_value($field).unwrap()
    };
}

// 辅助宏：根据类型选择合适的字符串解析方法
#[macro_export]
macro_rules! parse_param_by_type {
    ($params:expr, $key:expr, i32) => {
        get_required_i32_param(&$params, $key)?
    };
    ($params:expr, $key:expr, f64) => {
        ordered_float::OrderedFloat::from(get_required_f64_param(&$params, $key)?)
    };
    ($params:expr, $key:expr, f32) => {
        ordered_float::OrderedFloat::from(get_required_f32_param(&$params, $key)?)
    };
    ($params:expr, $key:expr, ordered_float::OrderedFloat<f64>) => {
        ordered_float::OrderedFloat::from(get_required_f64_param(&$params, $key)?)
    };
    ($params:expr, $key:expr, ordered_float::OrderedFloat<f32>) => {
        ordered_float::OrderedFloat::from(get_required_f32_param(&$params, $key)?)
    };
    ($params:expr, $key:expr, $other_type:ty) => {
        get_required_special_param::<$other_type>(&$params, $key)?
    };
}

// 辅助宏：根据类型选择合适的JSON解析方法
#[macro_export]
macro_rules! parse_json_param_by_type {
    ($config:expr, $key:expr, i32) => {
        $config
            .get($key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or(format!("缺少必需参数: {}", $key))?
    };
    ($config:expr, $key:expr, f64) => {
        ordered_float::OrderedFloat::from(
            $config
                .get($key)
                .and_then(|v| v.as_f64())
                .ok_or(format!("缺少必需参数: {}", $key))?,
        )
    };
    ($config:expr, $key:expr, f32) => {
        ordered_float::OrderedFloat::from(
            $config
                .get($key)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or(format!("缺少必需参数: {}", $key))?,
        )
    };
    ($config:expr, $key:expr, ordered_float::OrderedFloat<f64>) => {
        ordered_float::OrderedFloat::from(
            $config
                .get($key)
                .and_then(|v| v.as_f64())
                .ok_or(format!("缺少必需参数: {}", $key))?,
        )
    };
    ($config:expr, $key:expr, ordered_float::OrderedFloat<f32>) => {
        ordered_float::OrderedFloat::from(
            $config
                .get($key)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or(format!("缺少必需参数: {}", $key))?,
        )
    };
    ($config:expr, $key:expr, $other_type:ty) => {
        $config
            .get($key)
            .and_then(|v| v.as_str())
            .ok_or(format!("缺少必需参数: {}", $key))?
            .parse::<$other_type>()
            .map_err(|e| format!("{}参数解析失败: {}", $key, e))?
    };
}

// 组合宏：同时定义配置和输出结构体
#[macro_export]
macro_rules! define_indicator {
    (
        $indicator_name:ident,
        params => [$(($param:ident: $param_type:ty)),* $(,)?],
        output => [$(($output_field:ident: $output_type:ty)),* $(,)?] $(,)?
    ) => {
        // 生成配置结构体
        $crate::define_indicator_config!(
            $indicator_name,
            params => [$(($param: $param_type)),*]
        );

        // 生成输出结构体
        $crate::define_indicator_output!(
            $indicator_name,
            output => [$(($output_field: $output_type)),*]
        );
    };
}

#[macro_export]
// 为Indicator枚举创建所有trait方法的宏
macro_rules! impl_indicator {
    ($enum_name:ident, $($variant:ident),+) => {
        impl crate::market::QuantData for $enum_name {
            fn get_datetime(&self) -> crate::system::DateTimeUtc {
                match self {
                    $(
                        $enum_name::$variant(inner) => inner.get_datetime(),
                    )+
                }
            }
            fn to_json(&self) -> serde_json::Value {
                match self {
                    $(
                        $enum_name::$variant(inner) => crate::market::QuantData::to_json(inner),
                    )+
                }
            }

            fn to_list(&self) -> Vec<f64> {
                match self {
                    $(
                        $enum_name::$variant(inner) => crate::market::QuantData::to_list(inner),
                    )+
                }
            }

            fn to_json_with_time(&self) -> serde_json::Value {
                match self {
                    $(
                        $enum_name::$variant(inner) => crate::market::QuantData::to_json_with_time(inner),
                    )+
                }
            }

            fn get_value(&self, key: &str) -> Option<f64> {
                match self {
                    $(
                        $enum_name::$variant(inner) => crate::market::QuantData::get_value(inner, key),
                    )+
                }
            }
        }

        // impl $enum_name {


        //     fn get_datetime(&self) -> DateTime<Utc> {
        //         match self {
        //             $(
        //                 $enum_name::$variant(inner) => inner.datetime,
        //             )+
        //         }
        //     }

        //     fn get_timestamp(&self) -> i64 {
        //         match self {
        //             $(
        //                 $enum_name::$variant(inner) => inner.datetime.timestamp_millis(),
        //             )+
        //         }
        //     }

        //     fn to_json(&self) -> serde_json::Value {
        //         match self {
        //             $(
        //                 $enum_name::$variant(inner) => CacheItem::to_json(inner),
        //             )+
        //         }
        //     }

        //     fn to_list(&self) -> Vec<f64> {
        //         match self {
        //             $(
        //                 $enum_name::$variant(inner) => CacheItem::to_list(inner),
        //             )+
        //         }
        //     }

        //     fn to_json_with_time(&self) -> serde_json::Value {
        //         match self {
        //             $(
        //                 $enum_name::$variant(inner) => CacheItem::to_json_with_time(inner),
        //             )+
        //         }
        //     }
        // }
    };
}

#[macro_export]
macro_rules! impl_indicator_config {
    ($indicator_config_enum:ident, ($($indicator_name:ident),+ $(,)?)) => {
        paste::paste! {
            // 实现 ToString trait
            impl ToString for $indicator_config_enum {
                fn to_string(&self) -> String {
                    match self {
                        $(
                            $indicator_config_enum::$indicator_name(config) => config.to_string(),
                        )+
                    }
                }
            }

            // 实现 FromStr trait
            impl std::str::FromStr for $indicator_config_enum {
                type Err = StarRiverError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    // 提取指标类型（如"sma"）
                    let indicator_type = if s.contains("(") {
                        s.split("(").next().unwrap_or("").trim()
                    } else {
                        s
                    };

                    // 根据指标类型创建相应的配置
                    match indicator_type {
                        $(
                            stringify!([<$indicator_name:lower>]) | stringify!([<$indicator_name:snake>]) => Ok($indicator_config_enum::$indicator_name([<$indicator_name Config>]::from_str(s)?)),
                        )+
                        _ => Err(InvalidIndicatorTypeSnafu {indicator_type: indicator_type.to_string()}.build()),
                    }
                }
            }

            // 实现 new 方法
            impl $indicator_config_enum {
                pub fn new(indicator_type: &str, config: &serde_json::Value) -> Result<Self, IndicatorError> {
                    tracing::debug!("indicator_type: {:?}, config: {:?}", indicator_type, config);
                    match indicator_type {
                        $(
                            stringify!([<$indicator_name:lower>]) | stringify!([<$indicator_name:snake>]) => {
                                Ok($indicator_config_enum::$indicator_name([<$indicator_name Config>]::new(config).context(CreateIndicatorFailedSnafu {indicator_type: indicator_type.to_string()})?))
                            }
                        )+
                        _ => {
                            // use serde::de::Error as _;
                            // Err(serde_json::Error::custom(format!("匹配指标类型失败: {}", indicator_type)))
                            Err(UnsupportedIndicatorTypeSnafu {indicator_type: indicator_type.to_string()}.build())
                        },
                    }
                }
            }
        }
    };
}
