use std::collections::HashMap;

/// 解析指标配置字符串的通用工具函数
/// 
/// 输入格式: `indicator_name(param1=value1 param2=value2 ...)`
/// 
/// # 返回值
/// 返回一个元组 (指标名称, 参数HashMap)
/// 
/// # 示例
/// ```
/// use types::indicator::utils::parse_indicator_config_from_str;
/// let result = parse_indicator_config_from_str("macd(fast=12 slow=26 signal=9 source=close)");
/// // 返回: Ok(("macd", {"fast": "12", "slow": "26", "signal": "9", "source": "close"}))
/// ```
pub fn parse_indicator_config_from_str(s: &str) -> Result<(String, HashMap<String, String>), String> {
    // 验证格式并分离指标名称和参数部分
    let parts: Vec<&str> = s.split('(').collect();
    if parts.len() != 2 {
        return Err("指标配置格式无效，应为 'indicator_name(param1=value1 param2=value2 ...)'".to_string());
    }

    let indicator_name = parts[0].trim().to_lowercase();
    let content = parts[1].split(')').next().unwrap_or_default();
    if content.trim().is_empty() {
        return Err("指标配置参数不能为空".to_string());
    }

    let mut params = HashMap::new();

    // 按空格分割参数
    let param_parts: Vec<&str> = content.split_whitespace().collect();
    // 解析每个键值对
    for param in param_parts {
        let kv: Vec<&str> = param.split('=').collect();
        if kv.len() != 2 {
            return Err(format!("参数格式无效: {}，应为 'key=value' 格式", param));
        }
        
        let key = kv[0].trim().to_string();
        let value = kv[1].trim().to_string();
        
        if key.is_empty() || value.is_empty() {
            return Err(format!("参数键或值不能为空: {}", param));
        }
        
        params.insert(key, value);
    }

    Ok((indicator_name, params))
}

/// 从参数HashMap中获取必需的整数参数
pub fn get_required_i32_param(params: &HashMap<String, String>, key: &str) -> Result<i32, String> {
    let value = params.get(key).ok_or(format!("缺少必需参数: {}", key))?;
    value.parse::<i32>().map_err(|e| format!("{}参数解析失败: {}", key, e))
}

/// 从参数HashMap中获取必需的浮点数参数
pub fn get_required_f64_param(params: &HashMap<String, String>, key: &str) -> Result<f64, String> {
    let value = params.get(key).ok_or(format!("缺少必需参数: {}", key))?;
    value.parse::<f64>().map_err(|e| format!("{}参数解析失败: {}", key, e))
}

/// 从参数HashMap中获取必需的f32浮点数参数
pub fn get_required_f32_param(params: &HashMap<String, String>, key: &str) -> Result<f32, String> {
    let value = params.get(key).ok_or(format!("缺少必需参数: {}", key))?;
    value.parse::<f32>().map_err(|e| format!("{}参数解析失败: {}", key, e))
}

/// 从参数HashMap中获取必需的字符串参数并解析为指定类型
pub fn get_required_special_param<T>(params: &HashMap<String, String>, key: &str) -> Result<T, String> 
where 
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = params.get(key).ok_or(format!("缺少必需参数: {}", key))?;
    value.parse::<T>().map_err(|e| format!("{}参数解析失败: {}", key, e))
}