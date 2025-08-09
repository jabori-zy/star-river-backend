use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use types::indicator::indicator_define::price_transform::*;
use types::indicator::IndicatorTrait;
use crate::value::*;

/// 测试所有价格转换指标
pub fn test_price_transform_indicators() {
    println!("=== 价格转换指标测试 ===");
    
    // 准备测试数据
    // let datasets = generate_test_datasets();
    
    // for (name, kline_series) in datasets {
    //     println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());
    let kline_series = generate_simple_kline_series(100, None, None);
    // 测试所有价格转换指标
    test_avgprice(&kline_series);
    test_medprice(&kline_series);
    test_typprice(&kline_series);
    test_wclprice(&kline_series);
}

/// 测试AVGPRICE指标
fn test_avgprice(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_avgprice(kline_series.to_vec()) {
        Ok(results) => {
            println!("  AVGPRICE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  AVGPRICE: 计算失败 - {:?}", e),
    }
}

/// 测试MEDPRICE指标
fn test_medprice(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_medprice(kline_series.to_vec()) {
        Ok(results) => {
            println!("  MEDPRICE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MEDPRICE: 计算失败 - {:?}", e),
    }
}

/// 测试TYPPRICE指标
fn test_typprice(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_typprice(kline_series.to_vec()) {
        Ok(results) => {
            println!("  TYPPRICE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  TYPPRICE: 计算失败 - {:?}", e),
    }
}

/// 测试WCLPRICE指标
fn test_wclprice(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    match CalculateIndicatorFunction::calculate_wclprice(kline_series.to_vec()) {
        Ok(results) => {
            println!("  WCLPRICE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  WCLPRICE: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_transform_indicators_integration() {
        test_price_transform_indicators();
    }

    #[test]
    fn test_avgprice_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_avgprice(&kline_series);
    }

    #[test]
    fn test_typprice_individual() {
        let kline_series = generate_simple_kline_series(50, None, None);
        test_typprice(&kline_series);
    }

    #[test]
    fn test_price_transforms_with_different_data() {
        println!("测试不同数据集下的价格转换:");
        
        // 测试上升趋势数据
        let uptrend_data = generate_trending_kline_series(20, 1.0);
        println!("\n上升趋势数据:");
        test_avgprice(&uptrend_data);
        
        // 测试下降趋势数据
        let downtrend_data = generate_trending_kline_series(20, -0.5);
        println!("\n下降趋势数据:");
        test_avgprice(&downtrend_data);
    }
}