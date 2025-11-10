use std::collections::HashMap;

use snafu::ResultExt;

use crate::error::{
    ConfigMissParamSnafu, InvalidConfigFormatSnafu, ParamEmptySnafu, ParamFormatInvalidSnafu, ParseFloatParamFailedSnafu,
    ParseIntParamFailedSnafu, ParseSpecialParamFailedSnafu, TaLibError,
};

pub fn parse_indicator_config_from_str(s: &str) -> Result<(String, HashMap<String, String>), TaLibError> {
    // 验证格式并分离指标名称和参数部分
    let parts: Vec<&str> = s.split('(').collect();
    if parts.len() != 2 {
        return Err(InvalidConfigFormatSnafu {
            indicator_config: s.to_string(),
        }
        .build());
    }

    let indicator_name = parts[0].trim().to_lowercase();
    let content = parts[1].split(')').next().unwrap_or_default();
    if content.trim().is_empty() {
        return Err(ParamEmptySnafu {
            indicator_config: s.to_string(),
        }
        .build());
    }

    let mut params = HashMap::new();

    // 按空格分割参数
    let param_parts: Vec<&str> = content.split_whitespace().collect();
    // 解析每个键值对
    for param in param_parts {
        let kv: Vec<&str> = param.split('=').collect();
        if kv.len() != 2 {
            return Err(ParamFormatInvalidSnafu {
                indicator_config: s.to_string(),
            }
            .build());
        }

        let key = kv[0].trim().to_string();
        let value = kv[1].trim().to_string();

        if key.is_empty() || value.is_empty() {
            return Err(ParamFormatInvalidSnafu {
                indicator_config: s.to_string(),
            }
            .build());
        }

        params.insert(key, value);
    }

    Ok((indicator_name, params))
}

/// 从参数HashMap中获取必需的整数参数
pub fn get_required_i32_param(params: &HashMap<String, String>, key: &str) -> Result<i32, TaLibError> {
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<i32>().context(ParseIntParamFailedSnafu { param: key.to_string() })
}

/// 从参数HashMap中获取必需的浮点数参数
pub fn get_required_f64_param(params: &HashMap<String, String>, key: &str) -> Result<f64, TaLibError> {
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<f64>().context(ParseFloatParamFailedSnafu { param: key.to_string() })
}

/// 从参数HashMap中获取必需的f32浮点数参数
pub fn get_required_f32_param(params: &HashMap<String, String>, key: &str) -> Result<f32, TaLibError> {
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<f32>().context(ParseFloatParamFailedSnafu { param: key.to_string() })
}

/// 从参数HashMap中获取必需的字符串参数并解析为指定类型
pub fn get_required_special_param<T>(params: &HashMap<String, String>, key: &str) -> Result<T, TaLibError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<T>().map_err(|e| {
        ParseSpecialParamFailedSnafu {
            param: key.to_string(),
            reason: e.to_string(),
        }
        .build()
    })
}
