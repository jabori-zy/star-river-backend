use crate::value::*;
use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use types::indicator::indicator_define::cycle::*;
use types::indicator::{IndicatorTrait, PriceSource};

/// 测试所有Cycle指标
pub fn test_cycle_indicators() {
    println!("=== 周期指标测试 ===");

    // 准备测试数据
    let datasets = generate_test_datasets();

    for (name, kline_series) in datasets {
        println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());

        // 测试所有Cycle指标
        test_ht_dcperiod(&kline_series);
        test_ht_dcphase(&kline_series);
        test_ht_phasor(&kline_series);
        test_ht_sine(&kline_series);
        test_ht_trendmode(&kline_series);
    }
}

/// 测试HT_DCPERIOD指标
fn test_ht_dcperiod(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = HtDcperiodConfig {
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ht_dcperiod(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  HT_DCPERIOD: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  HT_DCPERIOD: 计算失败 - {:?}", e),
    }
}

/// 测试HT_DCPHASE指标
fn test_ht_dcphase(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = HtDcphaseConfig {
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ht_dcphase(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  HT_DCPHASE: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  HT_DCPHASE: 计算失败 - {:?}", e),
    }
}

/// 测试HT_PHASOR指标
fn test_ht_phasor(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = HtPhasorConfig {
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ht_phasor(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  HT_PHASOR: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  HT_PHASOR: 计算失败 - {:?}", e),
    }
}

/// 测试HT_SINE指标
fn test_ht_sine(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = HtSineConfig {
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ht_sine(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  HT_SINE: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  HT_SINE: 计算失败 - {:?}", e),
    }
}

/// 测试HT_TRENDMODE指标
fn test_ht_trendmode(kline_series: &[std::sync::Arc<types::cache::CacheValue>]) {
    let config = HtTrendmodeConfig {
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ht_trendmode(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  HT_TRENDMODE: 成功计算 {} 个结果", results.len());
            let result_list = results
                .iter()
                .map(|v| v.to_list())
                .collect::<Vec<Vec<f64>>>();
            println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  HT_TRENDMODE: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_cycle_indicators_integration() {
    //     test_cycle_indicators();
    // }

    #[test]
    fn test_ht_dcperiod_individual() {
        let kline_series = generate_simple_kline_series(70, None, None);
        test_ht_dcperiod(&kline_series);
    }

    #[test]
    fn test_ht_dcphase_individual() {
        let kline_series = generate_simple_kline_series(70, None, None);
        test_ht_dcphase(&kline_series);
    }

    #[test]
    fn test_ht_phasor_individual() {
        let kline_series = generate_simple_kline_series(70, None, None);
        test_ht_phasor(&kline_series);
    }

    #[test]
    fn test_ht_sine_individual() {
        let kline_series = generate_simple_kline_series(70, None, None);
        test_ht_sine(&kline_series);
    }

    #[test]
    fn test_ht_trendmode_individual() {
        let kline_series = generate_simple_kline_series(70, None, None);
        test_ht_trendmode(&kline_series);
    }
}
