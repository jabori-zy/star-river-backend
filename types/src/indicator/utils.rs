use std::collections::HashMap;
use crate::indicator::indicator::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_indicator_config_from_str() {
        // 测试正常解析
        let result = parse_indicator_config_from_str("macd(fast=12 slow=26 signal=9 source=close)");
        assert!(result.is_ok());
        let (name, params) = result.unwrap();
        assert_eq!(name, "macd");
        assert_eq!(params.get("fast"), Some(&"12".to_string()));
        assert_eq!(params.get("slow"), Some(&"26".to_string()));
        assert_eq!(params.get("signal"), Some(&"9".to_string()));
        assert_eq!(params.get("source"), Some(&"close".to_string()));

        // 测试SMA单参数
        let result = parse_indicator_config_from_str("sma(period=14)");
        assert!(result.is_ok());
        let (name, params) = result.unwrap();
        assert_eq!(name, "sma");
        assert_eq!(params.get("period"), Some(&"14".to_string()));

        // 测试格式错误
        let result = parse_indicator_config_from_str("invalid_format");
        assert!(result.is_err());

        // 测试空参数
        let result = parse_indicator_config_from_str("sma()");
        assert!(result.is_err());

        // 测试参数格式错误
        let result = parse_indicator_config_from_str("sma(period)");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_required_params() {
        let mut params = HashMap::new();
        params.insert("period".to_string(), "14".to_string());
        params.insert("deviation".to_string(), "2.0".to_string());

        // 测试获取整数参数
        let period = get_required_i32_param(&params, "period");
        assert_eq!(period.unwrap(), 14);

        // 测试获取浮点数参数
        let deviation = get_required_f64_param(&params, "deviation");
        assert_eq!(deviation.unwrap(), 2.0);

        // 测试缺少参数
        let result = get_required_i32_param(&params, "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_indicator_parsing_integration() {
        use crate::indicator::indicator::*;
        use std::str::FromStr;

        // 测试 SMA 解析
        let sma_result = MAConfig::from_str("sma(period=14)");
        assert!(sma_result.is_ok());
        let sma_config = sma_result.unwrap();
        assert_eq!(sma_config.time_period, 14);

        // 测试 MACD 解析
        let macd_result = MACDConfig::from_str("macd(fast=12 slow=26 signal=9 source=close)");
        assert!(macd_result.is_ok());
        let macd_config = macd_result.unwrap();
        assert_eq!(macd_config.fast_period, 12);
        assert_eq!(macd_config.slow_period, 26);
        assert_eq!(macd_config.signal_period, 9);

        // 测试 BBands 解析
        let bbands_result = BBandsConfig::from_str("bbands(period=20 dev_up=2.0 dev_down=2.0 source=close ma_type=sma)");
        assert!(bbands_result.is_ok());
        let bbands_config = bbands_result.unwrap();
        assert_eq!(bbands_config.time_period, 20);

        // 测试 RSI 解析
        let rsi_result = RSIConfig::from_str("rsi(period=14 source=close)");
        assert!(rsi_result.is_ok());
        let rsi_config = rsi_result.unwrap();
        assert_eq!(rsi_config.time_period, 14);

        // 测试错误情况
        let invalid_result = MAConfig::from_str("invalid_format");
        assert!(invalid_result.is_err());

        let missing_param_result = MAConfig::from_str("sma()");
        assert!(missing_param_result.is_err());
    }
}
