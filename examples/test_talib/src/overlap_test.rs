use crate::value::*;
use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use ordered_float::OrderedFloat;
use star_river_core::indicator::indicator_define::overlap::*;
use star_river_core::indicator::{IndicatorTrait, MAType, PriceSource};

/// 测试所有重叠指标
pub fn test_overlap_indicators() {
    println!("=== 重叠指标测试 ===");

    // 准备测试数据
    // let datasets = generate_test_datasets();

    // for (name, kline_series) in datasets {
    //     println!("\n--- 数据集: {} (长度: {}) ---", name, kline_series.len());
    let kline_series = generate_simple_kline_series(100, None, None);
    // 测试所有重叠指标
    test_bbands(&kline_series);
    test_dema(&kline_series);
    test_ema(&kline_series);
    test_ht_trendline(&kline_series);
    test_kama(&kline_series);
    test_ma(&kline_series);
    test_mama(&kline_series);
    test_midpoint(&kline_series);
    test_midprice(&kline_series);
    test_sar(&kline_series);
    test_sarext(&kline_series);
    test_sma(&kline_series);
    test_t3(&kline_series);
    test_tema(&kline_series);
    test_trima(&kline_series);
    test_wma(&kline_series);
}

/// 测试BBands指标
fn test_bbands(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = BBANDSConfig {
        time_period: 20,
        dev_up: OrderedFloat(2.0),
        dev_down: OrderedFloat(2.0),
        ma_type: MAType::SMA,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_bbands(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  BBANDS: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            // //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  BBANDS: 计算失败 - {:?}", e),
    }
}

/// 测试DEMA指标
fn test_dema(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = DEMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_dema(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  DEMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            // //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  DEMA: 计算失败 - {:?}", e),
    }
}

/// 测试EMA指标
fn test_ema(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = EMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ema(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  EMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            // //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  EMA: 计算失败 - {:?}", e),
    }
}

/// 测试HtTrendline指标
fn test_ht_trendline(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = HtTrendlineConfig {
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ht_trendline(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  HT_TRENDLINE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  HT_TRENDLINE: 计算失败 - {:?}", e),
    }
}

/// 测试KAMA指标
fn test_kama(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = KAMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_kama(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  KAMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  KAMA: 计算失败 - {:?}", e),
    }
}

/// 测试MA指标
fn test_ma(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = MAConfig {
        time_period: 14,
        ma_type: MAType::SMA,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_ma(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MA: 计算失败 - {:?}", e),
    }
}

/// 测试MAMA指标
fn test_mama(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = MAMAConfig {
        fast_limit: OrderedFloat(0.5),
        slow_limit: OrderedFloat(0.05),
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_mama(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MAMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MAMA: 计算失败 - {:?}", e),
    }
}

/// 测试MIDPOINT指标
fn test_midpoint(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = MIDPOINTConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_midpoint(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MIDPOINT: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MIDPOINT: 计算失败 - {:?}", e),
    }
}

/// 测试MIDPRICE指标
fn test_midprice(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = MIDPRICEConfig { time_period: 14 };

    match CalculateIndicatorFunction::calculate_midprice(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  MIDPRICE: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  MIDPRICE: 计算失败 - {:?}", e),
    }
}

/// 测试SAR指标
fn test_sar(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = SARConfig {
        acceleration: OrderedFloat(0.02),
        maximum: OrderedFloat(0.2),
    };

    match CalculateIndicatorFunction::calculate_sar(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  SAR: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  SAR: 计算失败 - {:?}", e),
    }
}

/// 测试SAREXT指标
fn test_sarext(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = SAREXTConfig {
        start_value: OrderedFloat(0.0),
        offset_on_reverse: OrderedFloat(0.0),
        acceleration_init_long: OrderedFloat(0.02),
        acceleration_long: OrderedFloat(0.02),
        acceleration_max_long: OrderedFloat(0.2),
        acceleration_init_short: OrderedFloat(0.02),
        acceleration_short: OrderedFloat(0.02),
        acceleration_max_short: OrderedFloat(0.2),
    };

    match CalculateIndicatorFunction::calculate_sarext(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  SAREXT: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  SAREXT: 计算失败 - {:?}", e),
    }
}

/// 测试SMA指标
fn test_sma(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = SMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_sma(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  SMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  SMA: 计算失败 - {:?}", e),
    }
}

/// 测试T3指标
fn test_t3(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = T3Config {
        time_period: 14,
        v_factor: OrderedFloat(0.7),
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_t3(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  T3: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  T3: 计算失败 - {:?}", e),
    }
}

/// 测试TEMA指标
fn test_tema(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = TEMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_tema(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  TEMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  TEMA: 计算失败 - {:?}", e),
    }
}

/// 测试TRIMA指标
fn test_trima(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = TRIMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_trima(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  TRIMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  TRIMA: 计算失败 - {:?}", e),
    }
}

/// 测试WMA指标
fn test_wma(kline_series: &[std::sync::Arc<star_river_core::cache::CacheValue>]) {
    let config = WMAConfig {
        time_period: 14,
        price_source: PriceSource::Close,
    };

    match CalculateIndicatorFunction::calculate_wma(kline_series.to_vec(), &config) {
        Ok(results) => {
            println!("  WMA: 成功计算 {} 个结果", results.len());
            let result_list = results.iter().map(|v| v.to_list()).collect::<Vec<Vec<f64>>>();
            //println!("    结果: {:?}", result_list);
        }
        Err(e) => println!("  WMA: 计算失败 - {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_indicators_integration() {
        test_overlap_indicators();
    }

    // #[test]
    // fn test_sma_individual() {
    //     let kline_series = generate_simple_kline_series(50, None, None);
    //     test_sma(&kline_series);
    // }

    // #[test]
    // fn test_ema_individual() {
    //     let kline_series = generate_simple_kline_series(50, None, None);
    //     test_ema(&kline_series);
    // }

    // #[test]
    // fn test_bbands_individual() {
    //     let kline_series = generate_simple_kline_series(50, None, None);
    //     test_bbands(&kline_series);
    // }
}
