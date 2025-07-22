/*!
# 指标宏使用指南

本模块提供了三个独立的宏来定义技术指标：

## 1. `define_indicator_config!` - 定义配置结构体
只生成指标的配置结构体及相关实现。

```rust
use crate::define_indicator_config;

define_indicator_config!(SMA,
    params => [(period: i32)]
);
```

## 2. `define_indicator_output!` - 定义输出结构体  
只生成指标的输出结构体及相关实现。

```rust
use crate::define_indicator_output;

define_indicator_output!(SMA,
    output => [(timestamp: i64), (value: f64)]
);
```

## 3. `define_indicator!` - 组合宏
同时生成配置和输出结构体，相当于调用上面两个宏。

```rust
use crate::define_indicator;

define_indicator!(SMA,
    params => [(period: i32)],
    output => [(timestamp: i64), (value: f64)]
);
```

这三个宏现在完全独立，可以根据需要单独使用。
*/

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

            impl crate::indicator::IndicatorTrait for $indicator_name {
                fn to_json(&self) -> serde_json::Value {
                    serde_json::to_value(self).unwrap()
                }

                fn to_list(&self) -> Vec<f64> {
                    let mut result = Vec::new();
                    $(
                        $crate::add_field_to_vec!(result, self.$output_field, $output_type);
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
            }

            impl crate::cache::CacheItem for $indicator_name {
                fn to_json(&self) -> serde_json::Value {
                    crate::indicator::IndicatorTrait::to_json(self)
                }

                fn to_list(&self) -> Vec<f64> {
                    crate::indicator::IndicatorTrait::to_list(self)
                }

                fn get_timestamp(&self) -> i64 {
                    self.timestamp
                }

                fn to_json_with_time(&self) -> serde_json::Value {
                    crate::indicator::IndicatorTrait::to_json_with_time(self)
                }
            }
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
                    format!("{}()", stringify!($indicator_name).to_lowercase())
                }
            }

            impl std::str::FromStr for [<$indicator_name Config>] {
                type Err = String;
                #[allow(unused_variables)]
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    // use crate::indicator::utils::*;
                    // let (_name, params) = parse_indicator_config_from_str(s)?;
                    Ok(Self {})
                }
            }

            impl crate::indicator::IndicatorConfigTrait for [<$indicator_name Config>] {
                #[allow(unused_variables)]
                fn new(config: &serde_json::Value) -> Result<Self, String> {
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
            pub struct [<$indicator_name Config>] {
                $(
                    pub $param: $crate::wrap_float_type!($param_type),
                )*
            }

            impl ToString for [<$indicator_name Config>] {
                fn to_string(&self) -> String {
                    let mut params: Vec<String> = Vec::new();
                    $(
                        params.push(format!("{}={:?}", stringify!($param), self.$param));
                    )*
                    format!("{}({})", stringify!($indicator_name).to_lowercase(), params.join(", "))
                }
            }

            impl std::str::FromStr for [<$indicator_name Config>] {
                type Err = String;
                
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
                fn new(config: &serde_json::Value) -> Result<Self, String> {
                    $(
                        let camel_field = crate::indicator::indicator_macros::snake_to_camel(stringify!($param));
                        let $param = $crate::parse_json_param_by_type!(config, &camel_field, $param_type);
                    )*
                    Ok(Self {
                        $($param),*
                    })
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
    ($vec:expr, $field:expr, f64) => {
        $vec.push($field);
    };
    ($vec:expr, $field:expr, f32) => {
        $vec.push($field as f64);
    };
    ($vec:expr, $field:expr, $other_type:ty) => {
        $vec.push($field as f64);
    };
}

// 辅助宏：格式化字段（用于to_json_with_time方法）
#[macro_export]
macro_rules! format_field_with_time {
    ($field:expr, $field_name:expr, i64) => {
        if $field_name == "timestamp" {
            serde_json::Value::String(utils::timestamp_to_utc8($field))
        } else {
            serde_json::Value::from($field)
        }
    };
    ($field:expr, $field_name:expr, $other_type:ty) => {
        serde_json::Value::from($field)
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
        $config.get($key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .ok_or(format!("缺少必需参数: {}", $key))?
    };
    ($config:expr, $key:expr, f64) => {
        ordered_float::OrderedFloat::from(
            $config.get($key)
                .and_then(|v| v.as_f64())
                .ok_or(format!("缺少必需参数: {}", $key))?
        )
    };
    ($config:expr, $key:expr, f32) => {
        ordered_float::OrderedFloat::from(
            $config.get($key)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or(format!("缺少必需参数: {}", $key))?
        )
    };
    ($config:expr, $key:expr, ordered_float::OrderedFloat<f64>) => {
        ordered_float::OrderedFloat::from(
            $config.get($key)
                .and_then(|v| v.as_f64())
                .ok_or(format!("缺少必需参数: {}", $key))?
        )
    };
    ($config:expr, $key:expr, ordered_float::OrderedFloat<f32>) => {
        ordered_float::OrderedFloat::from(
            $config.get($key)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .ok_or(format!("缺少必需参数: {}", $key))?
        )
    };
    ($config:expr, $key:expr, $other_type:ty) => {
        $config.get($key)
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
        impl IndicatorTrait for $enum_name {
            fn to_json(&self) -> serde_json::Value {
                match self {
                    $(
                        $enum_name::$variant(inner) => IndicatorTrait::to_json(inner),
                    )+
                }
            }

            fn to_list(&self) -> Vec<f64> {
                match self {
                    $(
                        $enum_name::$variant(inner) => IndicatorTrait::to_list(inner),
                    )+
                }
            }

            fn to_json_with_time(&self) -> serde_json::Value {
                match self {
                    $(
                        $enum_name::$variant(inner) => IndicatorTrait::to_json_with_time(inner),
                    )+
                }
            }
        }

        impl CacheItem for $enum_name {
            fn get_timestamp(&self) -> i64 {
                match self {
                    $(
                        $enum_name::$variant(inner) => inner.timestamp,
                    )+
                }
            }

            fn to_json(&self) -> serde_json::Value {
                match self {
                    $(
                        $enum_name::$variant(inner) => CacheItem::to_json(inner),
                    )+
                }
            }

            fn to_list(&self) -> Vec<f64> {
                match self {
                    $(
                        $enum_name::$variant(inner) => CacheItem::to_list(inner),
                    )+
                }
            }

            fn to_json_with_time(&self) -> serde_json::Value {
                match self {
                    $(
                        $enum_name::$variant(inner) => CacheItem::to_json_with_time(inner),
                    )+
                }
            }
        }
    };
}

#[macro_export]
/// 为IndicatorConfig枚举创建所有重复trait方法的宏
/// 
/// 使用方式：
/// ```rust
/// impl_indicator_config!(IndicatorConfig,
///     (MA, "ma"),
///     (MACD, "macd"),
///     (BBands, "bbands"),
///     (RSI, "rsi"),
///     (ADX, "adx")
/// );
/// ```
macro_rules! impl_indicator_config {
    ($enum_name:ident, ($($indicator_name:ident),+ $(,)?)) => {
        paste::paste! {
            // 实现 ToString trait
            impl ToString for $enum_name {
                fn to_string(&self) -> String {
                    match self {
                        $(
                            $enum_name::$indicator_name(config) => config.to_string(),
                        )+
                    }
                }
            }

            // 实现 FromStr trait
            impl std::str::FromStr for $enum_name {
                type Err = String;

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
                            stringify!([<$indicator_name:lower>]) => Ok($enum_name::$indicator_name([<$indicator_name Config>]::from_str(s)?)),
                        )+
                        _ => Err(format!("不支持的指标类型: {}", indicator_type)),
                    }
                }
            }

            // 实现 new 方法
            impl $enum_name {
                pub fn new(indicator_type: &str, config: &serde_json::Value) -> Result<Self, String> {
                    match indicator_type {
                        $(
                            stringify!([<$indicator_name:lower>]) => Ok($enum_name::$indicator_name([<$indicator_name Config>]::new(config)?)),
                        )+
                        _ => Err(format!("不支持的指标类型: {}", indicator_type)),
                    }
                }
            }
        }
    };
}