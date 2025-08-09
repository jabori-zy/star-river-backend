

use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use types::indicator::indicator_define::volume::*;
use types::indicator::IndicatorTrait;
use crate::value::*;

/// 测试所有成交量指标
pub fn test_volume_indicators() {
    println!("=== 成交量指标测试 ===");
    
    // 准备测试数据
    let datasets = generate_test_datasets();
    
    for (name, kline_series) in datasets {
        println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());
        
        // 测试所有成交量指标
        test_ad(&kline_series);
        test_adosc(&kline_series);
        test_obv(&kline_series);
    }
}

/// 测试AD指标 (Chaikin A/D Line - 钱德累积/派发线)
fn test_ad(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_ad(kline_series.to_vec()) {
        Ok(results) => {
            println!("  AD: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  AD: 计算失败 - {:?}", e),
    }
}

/// 测试ADOSC指标 (Chaikin A/D Oscillator - 钱德累积/派发振荡器)
fn test_adosc(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ADOSCConfig {
        fast_period: 3,
        slow_period: 10,
    };
    
    match CalculateIndicatorFunction::calculate_adosc(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ADOSC: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ADOSC: 计算失败 - {:?}", e),
    }
}

/// 测试OBV指标 (On Balance Volume - 能量潮)
fn test_obv(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_obv(kline_series.to_vec()) {
        Ok(results) => {
            println!("  OBV: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  OBV: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_indicators_integration() {
        test_volume_indicators();
    }

    #[test]
    fn test_ad_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_ad(&kline_series);
    }

    #[test]
    fn test_adosc_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_adosc(&kline_series);
    }

    #[test]
    fn test_obv_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_obv(&kline_series);
    }
}