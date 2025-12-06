use std::collections::HashMap;

use snafu::ResultExt;

use crate::error::{
    ConfigMissParamSnafu, InvalidConfigFormatSnafu, ParamEmptySnafu, ParamFormatInvalidSnafu, ParseFloatParamFailedSnafu,
    ParseIntParamFailedSnafu, ParseSpecialParamFailedSnafu, TaLibError,
};

pub fn parse_indicator_config_from_str(s: &str) -> Result<(String, HashMap<String, String>), TaLibError> {
    // Validate format and separate indicator name from parameters
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

    // Split parameters by whitespace
    let param_parts: Vec<&str> = content.split_whitespace().collect();
    // Parse each key-value pair
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

/// Get required integer parameter from HashMap
pub fn get_required_i32_param(params: &HashMap<String, String>, key: &str) -> Result<i32, TaLibError> {
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<i32>().context(ParseIntParamFailedSnafu { param: key.to_string() })
}

/// Get required float parameter from HashMap
pub fn get_required_f64_param(params: &HashMap<String, String>, key: &str) -> Result<f64, TaLibError> {
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<f64>().context(ParseFloatParamFailedSnafu { param: key.to_string() })
}

/// Get required f32 float parameter from HashMap
pub fn _get_required_f32_param(params: &HashMap<String, String>, key: &str) -> Result<f32, TaLibError> {
    let value = params.get(key).ok_or(ConfigMissParamSnafu { param: key.to_string() }.build())?;
    value.parse::<f32>().context(ParseFloatParamFailedSnafu { param: key.to_string() })
}

/// Get required string parameter from HashMap and parse to specified type
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
