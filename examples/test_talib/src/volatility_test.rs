use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use types::indicator::indicator_define::volatility::*;
use types::indicator::IndicatorTrait;
use crate::value::*;
/// 测试所有波动率指标
pub fn test_volatility_indicators() {
    println!("=== 波动率指标测试 ===");
    
    // 准备测试数据
    // let datasets = generate_test_datasets();
    
    // for (name, kline_series) in datasets {
    //     println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());
    

    // 测试所有波动率指标
    let kline_series = generate_simple_kline_series(100, None, None);
    test_atr(&kline_series);
    test_natr(&kline_series);
    test_trange(&kline_series);
    
}

/// 测试ATR指标
fn test_atr(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = ATRConfig {
        time_period: 14,
    };
    
    match CalculateIndicatorFunction::calculate_atr(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  ATR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  ATR: 计算失败 - {:?}", e),
    }
}

/// 测试NATR指标
fn test_natr(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = NATRConfig {
        time_period: 14,
    };
    
    match CalculateIndicatorFunction::calculate_natr(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  NATR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  NATR: 计算失败 - {:?}", e),
    }
}

/// 测试TRANGE指标
fn test_trange(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_trange(kline_series.to_vec()) {
        Ok(results) => {
            println!("  TRANGE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  TRANGE: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_indicators_integration() {
        test_volatility_indicators();
    }

    #[test]
    fn test_atr_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_atr(&kline_series);
    }

    #[test]
    fn test_natr_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_natr(&kline_series);
    }

    #[test]
    fn test_trange_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_trange(&kline_series);
    }



}